//! Minimal Server implementation serving as prototype

use crate::MediaRegistry;
use crate::api::{FromPluginEvent, ToPluginEvent};
use crate::authentication::Authenticator;
use crate::client_connection::ClientConnection;
use crate::world::map_block_router::ToRouterMessage;
use log::info;
use luanti_protocol::LuantiServer;
use luanti_protocol::types::NodeDefManager;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;

/// A server providing access to a single Luanti world
pub struct LuantiWorldServer {
    /// used to accept connection from clients
    bind_addr: SocketAddr,
    verbosity: u8,
    runner: Option<JoinHandle<()>>,
    node_def: Arc<NodeDefManager>,
    media: Arc<MediaRegistry>,
    plugin_event_sender: UnboundedSender<ToPluginEvent>,
    plugin_event_receiver: Option<UnboundedReceiver<FromPluginEvent>>,
}

impl LuantiWorldServer {
    /// Creates a new [`LuantiWorldServer`].
    #[must_use]
    pub fn new(
        bind_addr: SocketAddr,
        verbosity: u8,
        node_def: Arc<NodeDefManager>,
        media: Arc<MediaRegistry>,
        plugin_event_sender: UnboundedSender<ToPluginEvent>,
        plugin_event_receiver: UnboundedReceiver<FromPluginEvent>,
    ) -> Self {
        Self {
            bind_addr,
            verbosity,
            runner: None,
            node_def,
            media,
            plugin_event_sender,
            plugin_event_receiver: Some(plugin_event_receiver),
        }
    }

    /// Starts a runner task for the server which listens on the configured socket for incoming
    /// connections and then return immediately.
    ///
    /// # Panics
    ///
    /// Panics if the server is already running.
    pub fn start(
        &mut self,
        authenticator: impl Authenticator + 'static,
        block_interest_sender: UnboundedSender<ToRouterMessage>,
    ) {
        assert!(self.runner.is_none(), "server is already running");

        let bind_addr = self.bind_addr;
        let verbosity = self.verbosity;
        let node_def_clone = Arc::clone(&self.node_def);
        let media_clone = Arc::clone(&self.media);
        let runner = tokio::spawn(Self::accept_connections(
            bind_addr,
            authenticator,
            verbosity,
            block_interest_sender,
            node_def_clone,
            media_clone,
            self.plugin_event_sender.clone(),
            self.plugin_event_receiver.take().unwrap(),
        ));
        self.runner.replace(runner);
    }

    async fn accept_connections<Auth: Authenticator + 'static>(
        bind_addr: SocketAddr,
        authenticator: Auth,
        verbosity: u8,
        block_interest_sender: UnboundedSender<ToRouterMessage>,
        node_def: Arc<NodeDefManager>,
        media: Arc<MediaRegistry>,
        plugin_event_sender: UnboundedSender<ToPluginEvent>,
        from_plugin_event_receiver: UnboundedReceiver<FromPluginEvent>,
    ) {
        let mut server = LuantiServer::new(bind_addr);
        let mut connection_id = 1;

        #[expect(clippy::infinite_loop, reason = "// TODO add a cancellation mechanism")]
        loop {
            let connection = server.accept().await;

            let id = connection_id;
            connection_id += 1;
            info!(
                "[P{}] New client connected from {:?}",
                id,
                connection.remote_addr()
            );

            ClientConnection::spawn(
                id,
                connection,
                authenticator.clone(),
                verbosity,
                block_interest_sender.clone(),
                Arc::clone(&node_def),
                Arc::clone(&media),
                plugin_event_sender.clone(),
                from_plugin_event_receiver,
            );

            break;
        }
    }
}
