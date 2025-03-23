//!
//! Peer
//!
//! Turns a datagram stream (e.g. from a `UdpSocket`) into a stream
//! of Luanti Commands, and vice versa.
//!
//! This handles reliable transport, as well as packet splitting and
//! split packet reconstruction.
//!
//! This also handles control packets. In particular, it keeps track
//! of the assigned peer id and includes it on every packet.
//!  

mod channel;
mod reliable_receiver;
mod reliable_sender;
mod sequence_number;
mod split_receiver;
mod split_sender;

use anyhow::Result;
use anyhow::bail;
use channel::Channel;
use log::debug;
use log::error;
use log::info;
use log::trace;
use log::warn;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::mpsc::unbounded_channel;

use crate::commands::Command;
use crate::commands::CommandProperties;
use crate::commands::server_to_client::ToClientCommand;
use crate::types::ProtocolContext;
use crate::wire::channel_id::ChannelId;
use crate::wire::deser::Deserialize;
use crate::wire::deser::Deserializer;
use crate::wire::packet::AckBody;
use crate::wire::packet::ControlBody;
use crate::wire::packet::Packet;
use crate::wire::packet::PacketBody;
use crate::wire::packet::ReliableBody;
use crate::wire::packet::SetPeerIdBody;
use crate::wire::peer_id::PeerId;
use crate::wire::ser::Serialize;
use crate::wire::ser::VecSerializer;

use reliable_receiver::ReliableReceiver;
use reliable_sender::ReliableSender;
use split_receiver::SplitReceiver;
use split_sender::SplitSender;

use std::net::SocketAddr;
use std::time::Duration;
use std::time::Instant;

// How long to accept peer_id == 0 from a client after sending set_peer_id
const INEXISTENT_PEER_ID_GRACE: Duration = Duration::from_secs(20);

#[derive(thiserror::Error, Debug)]
pub enum PeerError {
    #[error("Peer sent disconnect packet")]
    PeerSentDisconnect,
    #[error("Socket Closed")]
    SocketClosed,
    #[error("Controller Closed")]
    ControllerClosed,
    #[error("Internal Peer error")]
    InternalPeerError,
}

pub type FullSeqNum = u64;

// This is held by the driver that interfaces with the LuantiSocket
pub struct Peer {
    remote_addr: SocketAddr,
    remote_is_server: bool,
    /// TODO(paradust): Add back-pressure
    send: UnboundedSender<Command>,
    recv: UnboundedReceiver<Result<Command>>,
}

impl Peer {
    #[must_use]
    pub fn remote_addr(&self) -> SocketAddr {
        self.remote_addr
    }

    /// Returns the is server of this [`Peer`].
    #[must_use]
    pub fn is_server(&self) -> bool {
        self.remote_is_server
    }

    /// Send command to peer
    /// If this fails, the peer has disconnected.
    pub fn send(&self, command: Command) -> Result<()> {
        self.send.send(command)?;
        Ok(())
    }

    /// Receive command from the peer
    /// Returns (channel, reliable flag, Command)
    /// If this fails, the peer is disconnected.
    pub async fn recv(&mut self) -> Result<Command> {
        match self.recv.recv().await {
            Some(result) => result,
            None => bail!(PeerError::InternalPeerError),
        }
    }
}

// This is owned by the LuantiSocket
pub struct PeerIO {
    relay: UnboundedSender<SocketToPeer>,
}

#[must_use]
pub fn new_peer(
    remote_addr: SocketAddr,
    remote_is_server: bool,
    peer_to_socket: UnboundedSender<PeerToSocket>,
) -> (Peer, PeerIO) {
    let (peer_send_tx, peer_send_rx) = unbounded_channel();
    let (peer_recv_tx, peer_recv_rx) = unbounded_channel();
    let (relay_tx, relay_rx) = unbounded_channel();

    let socket_peer = Peer {
        remote_addr,
        remote_is_server,
        send: peer_send_tx,
        recv: peer_recv_rx,
    };
    let socket_peer_io = PeerIO { relay: relay_tx };
    let socket_peer_runner = PeerRunner {
        remote_addr,
        remote_is_server,
        recv_context: ProtocolContext::latest_for_receive(remote_is_server),
        send_context: ProtocolContext::latest_for_send(remote_is_server),
        connect_time: Instant::now(),
        remote_peer_id: PeerId::NONE,
        local_peer_id: PeerId::NONE,
        from_socket: relay_rx,
        from_controller: peer_send_rx,
        to_controller: peer_recv_tx.clone(),
        to_socket: peer_to_socket,
        channels: vec![
            Channel::new(remote_is_server, peer_recv_tx.clone()),
            Channel::new(remote_is_server, peer_recv_tx.clone()),
            Channel::new(remote_is_server, peer_recv_tx.clone()),
        ],
        now: Instant::now(),
        last_received: Instant::now(),
    };
    tokio::spawn(async move { socket_peer_runner.run().await });
    (socket_peer, socket_peer_io)
}

impl PeerIO {
    /// Parse the packet and send it to the runner
    /// Called by the `LuantiSocket` when a packet arrives for us
    ///
    pub fn send(&mut self, data: &[u8]) {
        //TODO Add back-pressure
        self.relay
            .send(SocketToPeer::Received(data.to_vec()))
            .unwrap_or_else(|error| {
                // TODO clarify error condition and handling
                error!("failed to relay packet: {error}");
            });
    }
}

#[derive(Debug)]
pub enum SocketToPeer {
    /// TODO(paradust): Use buffer pool
    Received(Vec<u8>),
}

#[derive(Debug)]
pub enum PeerToSocket {
    // Acks are sent with higher priority
    SendImmediate(SocketAddr, Vec<u8>),
    Send(SocketAddr, Vec<u8>),
    PeerIsDisconnected(SocketAddr),
}

pub struct PeerRunner {
    remote_addr: SocketAddr,
    remote_is_server: bool,
    connect_time: Instant,
    recv_context: ProtocolContext,
    send_context: ProtocolContext,

    // TODO(paradust): These should have a limited size, and close connection on overflow.
    from_socket: UnboundedReceiver<SocketToPeer>,
    to_socket: UnboundedSender<PeerToSocket>,

    // TODO(paradust): These should have back-pressure
    from_controller: UnboundedReceiver<Command>,
    to_controller: UnboundedSender<Result<Command>>,

    // This is the peer id in the Luanti protocol
    // Luanti's server uses these to keep track of clients, but we use the remote_addr.
    // Just use a randomly generated, not necessarily unique value, and keep it consistent.
    // Special ids: 0 is unassigned, and 1 for the server.
    remote_peer_id: PeerId,
    local_peer_id: PeerId,

    channels: Vec<Channel>,

    // Updated once per wakeup, to limit number of repeated sys-calls
    now: Instant,

    // Time last packet was received. Used to timeout connection.
    last_received: Instant,
}

impl PeerRunner {
    pub fn update_now(&mut self) {
        self.now = Instant::now();
        self.channels
            .iter_mut()
            .for_each(|channel| channel.update_now(&self.now));
    }

    pub fn serialize_for_send(&mut self, channel: ChannelId, body: PacketBody) -> Result<Vec<u8>> {
        let pkt = Packet::new(self.local_peer_id, channel, body);
        let mut serializer = VecSerializer::new(self.send_context, 512);
        Packet::serialize(&pkt, &mut serializer)?;
        Ok(serializer.take())
    }

    pub fn send_raw(&mut self, channel: ChannelId, body: PacketBody) -> Result<()> {
        let raw = self.serialize_for_send(channel, body)?;
        self.to_socket
            .send(PeerToSocket::Send(self.remote_addr, raw))?;
        Ok(())
    }

    pub fn send_raw_priority(&mut self, channel: ChannelId, body: PacketBody) -> Result<()> {
        let raw = self.serialize_for_send(channel, body)?;
        self.to_socket
            .send(PeerToSocket::SendImmediate(self.remote_addr, raw))?;
        Ok(())
    }

    pub async fn run(mut self) {
        if let Err(err) = self.run_inner().await {
            // Top-level error handling for a peer.
            // If an error gets to this point, the peer is toast.
            // Send a disconnect packet, and a remove peer request to the socket
            // These channels might already be dead, so ignore any errors.
            let disconnected_cleanly = if let Some(error) = err.downcast_ref::<PeerError>() {
                matches!(error, PeerError::PeerSentDisconnect)
            } else {
                false
            };
            if !disconnected_cleanly {
                // Send a disconnect packet
                #[expect(
                    clippy::unwrap_used,
                    reason = "// TODO clarify error condition and handling"
                )]
                self.send_raw(
                    ChannelId::Default,
                    (ControlBody::Disconnect).into_inner().into_unreliable(),
                )
                .unwrap();
            }
            #[expect(
                clippy::unwrap_used,
                reason = "// TODO clarify error condition and handling"
            )]
            self.to_socket
                .send(PeerToSocket::PeerIsDisconnected(self.remote_addr))
                .unwrap();

            // Tell the controller why we died
            self.to_controller.send(Err(err)).unwrap_or_else(|err| {
                // This might fail if the controller has been disconnected already
                // ignore the error in this case
                debug!("controller is no longer available: {err}");
            });
        }
    }

    pub async fn run_inner(&mut self) -> Result<()> {
        self.update_now();

        // 10 years ought to be enough
        let never = self.now + Duration::from_secs(315_576_000);

        loop {
            // Before select, make sure everything ready to send has been sent,
            // and compute a resend timeout.
            let mut next_wakeup = never;
            for channel_id in ChannelId::all() {
                loop {
                    let pkt = self.channels[usize::from(channel_id)].next_send(self.now);
                    match pkt {
                        Some(body) => self.send_raw(channel_id, body)?,
                        None => break,
                    }
                }
                if let Some(timeout) = self.channels[usize::from(channel_id)].next_timeout() {
                    next_wakeup = std::cmp::min(next_wakeup, timeout);
                }
            }

            // rust-analyzer chokes on code inside select!, so keep it to a minimum.
            tokio::select! {
                msg = self.from_socket.recv() => self.handle_from_socket(msg)?,
                command = self.from_controller.recv() => self.handle_from_controller(command)?,
                () = tokio::time::sleep_until(next_wakeup.into()) => self.handle_timeout()?,
            }
        }
    }

    fn handle_from_socket(&mut self, msg: Option<SocketToPeer>) -> Result<()> {
        self.update_now();
        let Some(msg) = msg else {
            bail!(PeerError::SocketClosed);
        };
        match msg {
            SocketToPeer::Received(buf) => {
                trace!(
                    "received {} bytes from socket: {:?}",
                    buf.len(),
                    &buf[0..buf.len().min(64)]
                );
                let mut deser = Deserializer::new(self.recv_context, &buf);
                let pkt = Packet::deserialize(&mut deser)?;
                self.last_received = self.now;
                self.process_packet(pkt)?;
            }
        };
        Ok(())
    }

    fn handle_from_controller(&mut self, command: Option<Command>) -> Result<()> {
        trace!("received command from controller: {command:?}",);

        self.update_now();
        let Some(command) = command else {
            bail!(PeerError::ControllerClosed);
        };
        self.sniff_hello(&command);

        self.send_command(command)?;
        Ok(())
    }

    fn handle_timeout(&mut self) -> Result<()> {
        self.update_now();
        self.process_timeouts()?;
        Ok(())
    }

    // Process a packet received over network
    fn process_packet(&mut self, pkt: Packet) -> Result<()> {
        if self.remote_is_server {
            if !pkt.sender_peer_id.is_server() {
                warn!("Server sending from wrong peer id");
                return Ok(());
            }
        } else {
            // We're the server, assign the remote a peer_id.
            if self.remote_peer_id.is_none() {
                // Assign a peer id
                self.local_peer_id = PeerId::SERVER;
                // FIXME this may hand out peer ids that are already in use
                self.remote_peer_id = PeerId::random();

                // Tell the client about it
                let set_peer_id = SetPeerIdBody::new(self.remote_peer_id).into_inner();
                self.channels[0].send_inner(true, set_peer_id);
            }
            if pkt.sender_peer_id.is_none() {
                if self.now > self.connect_time + INEXISTENT_PEER_ID_GRACE {
                    // Malformed, ignore.
                    warn!("Ignoring peer_id 0 packet");
                    return Ok(());
                }
            } else if pkt.sender_peer_id != self.remote_peer_id {
                // Malformed. Ignore
                warn!("Invalid peer_id on packet");
                return Ok(());
            }
        }

        // Send ack right away
        if let Some(rb) = pkt.as_reliable() {
            self.send_ack(pkt.channel, rb)?;
        }

        // Certain control packets need to be handled at the
        // top-level (here) instead of in a channel.
        // With the exception of disconnect, control packets must still be
        // passed to the channel, because they may have reliable bodies
        // (and affect seqnums)
        if let Some(control) = pkt.as_control() {
            #[expect(clippy::match_same_arms, reason = "for better documentation")]
            match control {
                ControlBody::Ack(_) => {
                    // Handled by channel
                }
                ControlBody::SetPeerId(set_peer_id) => {
                    if self.remote_is_server {
                        if self.local_peer_id.is_none() {
                            self.local_peer_id = set_peer_id.peer_id;
                        } else if self.local_peer_id != set_peer_id.peer_id {
                            bail!("Peer id mismatch in duplicate SetPeerId");
                        }
                    } else {
                        bail!("Invalid set_peer_id received from client");
                    }
                }
                ControlBody::Ping => {
                    // no-op. Packet already updated timeout
                }
                ControlBody::Disconnect => bail!(PeerError::PeerSentDisconnect),
            }
        }
        // If this is a HELLO packet, sniff it to set our protocol context.
        if let Some(command) = pkt.body.command() {
            self.sniff_hello(command);
        }

        self.channels[usize::from(pkt.channel)].process(pkt.body)
    }

    fn sniff_hello(&mut self, command: &Command) {
        if let Command::ToClient(ToClientCommand::Hello(spec)) = command {
            info!(
                "Server protocol version {} / serialization version {}",
                spec.proto_ver, spec.serialization_ver
            );
            self.update_context(spec.serialization_ver, spec.proto_ver);
        }
    }

    fn update_context(&mut self, ser_fmt: u8, protocol_version: u16) {
        self.recv_context.protocol_version = protocol_version;
        self.recv_context.ser_fmt = ser_fmt;
        self.send_context.protocol_version = protocol_version;
        self.send_context.ser_fmt = ser_fmt;
        self.channels
            .iter_mut()
            .for_each(|channel| channel.update_context(self.recv_context, self.send_context));
    }

    /// If this is a reliable packet, send an ack right away
    /// using a higher-priority out-of-band channel.
    fn send_ack(&mut self, channel: ChannelId, rb: &ReliableBody) -> Result<()> {
        let ack = AckBody::new(rb.seqnum).into_inner().into_unreliable();
        self.send_raw_priority(channel, ack)?;
        Ok(())
    }

    /// Send command to remote
    fn send_command(&mut self, command: Command) -> Result<()> {
        let channel = command.default_channel();
        let reliable = command.default_reliability();
        self.channels[usize::from(channel)].send(reliable, command)
    }

    #[expect(
        clippy::unused_self,
        clippy::unnecessary_wraps,
        reason = "// TODO this implementation looks incomplete"
    )]
    fn process_timeouts(&mut self) -> Result<()> {
        Ok(())
    }
}
