use std::collections::HashMap;
use std::collections::VecDeque;
use std::io::Error;
use std::net::SocketAddr;

use log::debug;
use log::error;
use tokio::io::Interest;
use tokio::io::Ready;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::unbounded_channel;

use crate::peer::PeerToSocket;

use crate::peer::Peer;
use crate::peer::PeerIO;
use crate::peer::new_peer;

const MAX_DATAGRAM_SIZE: usize = 0x0001_0000;

///
/// `LuantiSocket`
///
/// Handles the raw UDP socket, protocol validation, separating packets by peer,
/// reliable packet send, and split packets.
///
/// The actual contents of the communication, including authentication/handshaking,
/// are not handled at this layer.
///
pub struct LuantiSocket {
    accept_rx: UnboundedReceiver<Peer>,
    knock_tx: UnboundedSender<SocketAddr>,
    for_server: bool,
}

impl LuantiSocket {
    /// Create a new `LuantiSocket` and bind to address.
    /// The address may be V4 or V6.
    /// To select a random bind port, use 0.0.0.0:0 or [::]:0
    pub async fn new(bind_addr: SocketAddr, for_server: bool) -> Result<Self, Error> {
        let socket = UdpSocket::bind(bind_addr).await?;
        let (peer_tx, peer_rx) = unbounded_channel();
        let (accept_tx, accept_rx) = unbounded_channel();
        let (knock_tx, knock_rx) = unbounded_channel();
        let luanti_socket = Self {
            accept_rx,
            knock_tx,
            for_server,
        };
        let luanti_socket_runner = LuantiSocketRunner {
            socket,
            peers: HashMap::new(),
            peer_tx,
            peer_rx,
            outgoing: VecDeque::new(),
            accept_tx,
            knock_rx,
            for_server,
        };
        tokio::spawn(async move { luanti_socket_runner.run().await });
        Ok(luanti_socket)
    }

    /// Returns None when the server has shutdown.
    pub async fn accept(&mut self) -> Option<Peer> {
        self.accept_rx.recv().await
    }

    // Add a peer (server) manually. There is no network I/O.
    //
    // NOTE: This is not cancel safe, and it should not
    // be used if incoming connections are expected, or else
    // they will be discarded.
    pub async fn add_server(&mut self, server_address: SocketAddr) -> Peer {
        assert!(
            !self.for_server,
            "method my only be called for client sockets"
        );
        debug!("adding server as peer: {server_address}");
        self.knock_tx.send(server_address).unwrap();

        // Wait for the peer
        loop {
            let peer = self.accept().await.unwrap();
            let remote_address = peer.remote_addr();
            if remote_address == server_address {
                debug!("peer responded from remote address: {remote_address}");
                return peer;
            }
            debug!("ignoring random connect from another address: {remote_address}",);
        }
    }
}

pub struct LuantiSocketRunner {
    socket: UdpSocket,
    peers: HashMap<SocketAddr, PeerIO>,
    peer_tx: UnboundedSender<PeerToSocket>,
    peer_rx: UnboundedReceiver<PeerToSocket>,
    outgoing: VecDeque<(SocketAddr, Vec<u8>)>,
    accept_tx: UnboundedSender<Peer>,
    knock_rx: UnboundedReceiver<SocketAddr>,
    for_server: bool,
}

impl LuantiSocketRunner {
    pub async fn run(mut self) {
        // Top-level error handler
        match self.run_inner().await {
            Ok(()) => (),
            Err(err) => {
                error!("LuantiSocket abnormal exit: {err:?}");
            }
        }
    }

    pub async fn run_inner(&mut self) -> anyhow::Result<()> {
        let mut knock_closed = false;
        let mut buf = vec![0_u8; MAX_DATAGRAM_SIZE];

        loop {
            let interest = if self.outgoing.is_empty() {
                Interest::READABLE
            } else {
                Interest::READABLE | Interest::WRITABLE
            };
            // rust-analyzer chokes on code inside select!, so keep it to a minimum.
            tokio::select! {
                ready = self.socket.ready(interest) => self.handle_socket_io(ready, &mut buf),
                msg = self.peer_rx.recv() => self.handle_peer_message(msg),
                address = self.knock_rx.recv(), if !knock_closed => {
                    match address {
                        Some(address) => {
                            self.get_peer(address, true);
                        },
                        None => {
                            knock_closed = true;
                        },
                    }
                }
            }
        }
    }

    fn handle_socket_io(&mut self, ready: tokio::io::Result<Ready>, buf: &mut [u8]) {
        let ready = ready.expect("socket.ready should not error");
        if ready.is_readable() {
            match self.socket.try_recv_from(buf) {
                Ok((n, remote_addr)) => {
                    if let Some(peer) = self.get_peer(remote_addr, self.for_server) {
                        // TODO: If the peer receive channel is full, generate a disconnect message.
                        peer.send(&buf[..n]);
                    }
                }
                Err(ref error) if error.kind() == std::io::ErrorKind::WouldBlock => (),
                Err(error) => panic!("Unexpected socket error: {error:?}"),
            }
        }
        if ready.is_writable() && !self.outgoing.is_empty() {
            let (addr, data) = self.outgoing.pop_back().unwrap();
            match self.socket.try_send_to(&data, addr) {
                Ok(_) => (),
                Err(ref error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    self.outgoing.push_back((addr, data));
                }
                Err(error) => panic!("Unexpected socket error: {error:?}"),
            }
        }
    }

    fn handle_peer_message(&mut self, msg: Option<PeerToSocket>) {
        let Some(msg) = msg else {
            panic!("Unexpected Server shutdown?");
        };
        match msg {
            PeerToSocket::SendImmediate(addr, data) => self.outgoing.push_back((addr, data)),
            PeerToSocket::Send(addr, data) => self.outgoing.push_front((addr, data)),
            PeerToSocket::PeerIsDisconnected(addr) => self.remove_peer(addr),
        }
    }

    fn get_peer(&mut self, remote_addr: SocketAddr, may_insert: bool) -> Option<&mut PeerIO> {
        if may_insert && !self.peers.contains_key(&remote_addr) {
            self.insert_peer(remote_addr);
        }
        self.peers.get_mut(&remote_addr)
    }

    fn insert_peer(&mut self, remote_addr: SocketAddr) {
        let (peer, peer_io) = new_peer(remote_addr, !self.for_server, self.peer_tx.clone());
        self.peers.insert(remote_addr, peer_io);
        self.accept_tx.send(peer).unwrap();
    }

    fn remove_peer(&mut self, remote_addr: SocketAddr) {
        self.peers.remove(&remote_addr);
    }
}
