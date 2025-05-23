//! A single client connection to the server

mod authenticating;
mod loading;
mod running;
mod setup;
mod uninitialized;

use std::sync::Arc;

use crate::MediaRegistry;
use crate::authentication::Authenticator;
use crate::world::WorldBlock;
use crate::world::WorldUpdate;
use crate::world::map_block_router::ToRouterMessage;
use crate::world::view_tracker::ViewTracker;
use anyhow::Result;
use anyhow::anyhow;
use authenticating::AuthenticatingState;
use flexstr::SharedStr;
use loading::LoadingState;
use log::debug;
use log::error;
use log::info;
use log::trace;
use luanti_protocol::CommandDirection;
use luanti_protocol::CommandRef;
use luanti_protocol::LuantiConnection;
use luanti_protocol::commands::client_to_server::ToServerCommand;
use luanti_protocol::commands::server_to_client::BlockdataSpec;
use luanti_protocol::commands::server_to_client::ToClientCommand;
use luanti_protocol::peer::PeerError;
use luanti_protocol::types::MapNodesBulk;
use luanti_protocol::types::NodeDefManager;
use luanti_protocol::types::NodeMetadataList;
use luanti_protocol::types::TransferrableMapBlock;
use running::RunningState;
use setup::SetupState;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use uninitialized::UninitializedState;

pub(crate) struct ClientConnection<Auth: Authenticator> {
    id: u64,
    connection: LuantiConnection,
    verbosity: u8,
    state: State<Auth>,
    language: Option<String>,
    player_key: SharedStr,
    block_interest_sender: Option<mpsc::UnboundedSender<ToRouterMessage>>,
    world_update_sender: Option<mpsc::UnboundedSender<WorldUpdate>>,
    world_update_receiver: mpsc::UnboundedReceiver<WorldUpdate>,
    node_def: Arc<NodeDefManager>,
    media: Arc<MediaRegistry>,
}

impl<Auth: Authenticator + 'static> ClientConnection<Auth> {
    pub(crate) fn spawn(
        id: u64,
        connection: LuantiConnection,
        authenticator: Auth,
        verbosity: u8,
        block_interest_sender: mpsc::UnboundedSender<ToRouterMessage>,
        node_def: Arc<NodeDefManager>,
        media: Arc<MediaRegistry>,
    ) -> JoinHandle<()> {
        let (world_update_sender, world_update_receiver) = mpsc::unbounded_channel();

        let runner = ClientConnection {
            id,
            connection,
            verbosity,
            state: State::Uninitialized(UninitializedState::new(authenticator)),
            language: None,
            block_interest_sender: Some(block_interest_sender),
            player_key: SharedStr::EMPTY,
            world_update_sender: Some(world_update_sender),
            world_update_receiver,
            node_def,
            media,
        };
        tokio::spawn(async move { runner.run().await })
    }

    async fn run(mut self) {
        debug!("starting Luanti server runner");
        #[expect(
            clippy::large_futures,
            reason = "// TODO(kawogi) check whether a refactoring of the state machine can bring this down to normal"
        )]
        match self.run_inner().await {
            Ok(()) => (),
            Err(err) => {
                let show_err = if let Some(err) = err.downcast_ref::<PeerError>() {
                    !matches!(err, PeerError::PeerSentDisconnect)
                } else {
                    true
                };
                if show_err {
                    error!("[{}] Disconnected: {:?}", self.id, err);
                } else {
                    info!("[{}] Disconnected", self.id);
                }
            }
        }
    }

    async fn run_inner(&mut self) -> Result<()> {
        loop {
            // TODO(kawogi) review whether this should be refactored; all state transitions seem to be expressible as a simple sequence and do not require a full-fledged state machine
            tokio::select! {
                message = self.connection.recv() => {
                    trace!("connection.recv: {message:?}");
                    let message = message?;
                    self.maybe_show(&message);
                    self.handle_client_message(message).await?;
                },
                message = self.world_update_receiver.recv() => {
                    trace!("world_update_receiver.recv: {message:?}");
                    let Some(message) = message else {
                        anyhow::bail!("world update sender has been disconnected");
                    };
                    self.handle_world_update(message)?;
                }
            }
        }
    }

    async fn handle_client_message(&mut self, message: ToServerCommand) -> Result<()> {
        match &mut self.state {
            State::Uninitialized(state) => {
                if state.handle_message(message, &self.connection).await? {
                    debug!(
                        "initialization successfully completed; switching to authentication mode"
                    );
                    let next_state = state.next();
                    self.player_key = next_state.player_key().into();
                    self.state = State::Authenticating(next_state);
                } else {
                    debug!("initialization is still incomplete");
                }
            }
            State::Authenticating(state) => {
                if state.handle_message(message, &self.connection)? {
                    debug!("authentication successfully completed; switching to setup mode");
                    self.state = State::Setup(SetupState::new());
                } else {
                    debug!("authentication is still incomplete");
                }
            }
            State::Setup(state) => {
                if state.handle_message(message) {
                    debug!("setup successfully completed; switching to loading mode");
                    let next_state = state.next(Arc::clone(&self.media));
                    self.language = next_state.language().cloned();
                    self.state = State::Loading(next_state);

                    let State::Loading(loading_state) = &mut self.state else {
                        // this construction ensures that `self.state` is up to date _before_
                        // sending out all media to the client
                        unreachable!();
                    };
                    loading_state.send_data(&self.connection, &self.node_def, &self.media)?;
                } else {
                    debug!("setup is still incomplete");
                }
            }
            State::Loading(state) => {
                if state.handle_message(message, &self.connection)? {
                    debug!("loading successfully completed; switching to authenticated mode");

                    let block_interest_sender = self
                        .block_interest_sender
                        .take()
                        .ok_or(anyhow!("tried to take block_interest_sender twice"))?;
                    let world_update_sender = self
                        .world_update_sender
                        .take()
                        .ok_or(anyhow!("tried to take world_update_sender twice"))?;
                    let view_tracker = ViewTracker::new(
                        self.player_key.clone(),
                        block_interest_sender,
                        world_update_sender,
                    )?;

                    self.state = State::Running(RunningState::new(view_tracker));
                } else {
                    debug!("loading is still incomplete");
                }
            }
            State::Running(state) => state.handle_message(message, &self.connection)?,
        }

        Ok(())
    }

    fn is_bulk_command<Cmd: CommandRef>(command: &Cmd) -> bool {
        matches!(
            command.toclient_ref(),
            Some(ToClientCommand::Blockdata(_) | ToClientCommand::Media(_))
        )
    }

    fn maybe_show<Cmd: CommandRef>(&self, command: &Cmd) {
        let dir = match command.direction() {
            CommandDirection::ToClient => "S->C",
            CommandDirection::ToServer => "C->S",
        };
        let prefix = format!("[{}] {} ", self.id, dir);
        let mut verbosity = self.verbosity;
        if verbosity == 2 && Self::is_bulk_command(command) {
            // Show the contents of smaller commands, but skip the huge ones
            verbosity = 1;
        }
        match verbosity {
            0 => (),
            1 => trace!("{} {}", prefix, command.command_name()),
            2.. => trace!("{prefix} {command:#?}"),
        }
    }

    fn handle_world_update(&mut self, update: WorldUpdate) -> Result<()> {
        match update {
            WorldUpdate::NewMapBlock(world_block) => {
                let WorldBlock {
                    version: _,
                    pos: _,
                    is_underground,
                    day_night_differs,
                    lighting_complete,
                    nodes,
                    metadata,
                } = world_block;

                self.connection
                    .send(ToClientCommand::Blockdata(Box::new(BlockdataSpec {
                        pos: world_block.pos.vec(),
                        block: TransferrableMapBlock {
                            is_underground,
                            day_night_differs,
                            generated: true,
                            lighting_complete: Some(lighting_complete),
                            nodes: MapNodesBulk { nodes: nodes.0 },
                            node_metadata: NodeMetadataList { metadata },
                        },
                        network_specific_version: 2,
                    })))
                //
            }
        }
    }
}

enum State<Auth: Authenticator> {
    Uninitialized(UninitializedState<Auth>),
    Authenticating(AuthenticatingState),
    Setup(SetupState),
    Loading(LoadingState),
    Running(RunningState),
}
