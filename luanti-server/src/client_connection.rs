//! A single client connection to the server

mod authenticating;
mod loading;
mod running;
mod setup;
mod uninitialized;

use crate::authentication::Authenticator;
use anyhow::Result;
use authenticating::AuthenticatingState;
use loading::LoadingState;
use log::debug;
use log::error;
use log::info;
use log::trace;
use luanti_protocol::CommandDirection;
use luanti_protocol::CommandRef;
use luanti_protocol::LuantiConnection;
use luanti_protocol::commands::client_to_server::ToServerCommand;
use luanti_protocol::commands::server_to_client::ToClientCommand;
use luanti_protocol::peer::PeerError;
use running::RunningState;
use setup::SetupState;
use tokio::task::JoinHandle;
use uninitialized::UninitializedState;

pub(crate) struct ClientConnection<Auth: Authenticator> {
    id: u64,
    connection: LuantiConnection,
    verbosity: u8,
    state: State<Auth>,
    language: Option<String>,
}

impl<Auth: Authenticator + 'static> ClientConnection<Auth> {
    pub(crate) fn spawn(
        id: u64,
        connection: LuantiConnection,
        authenticator: Auth,
        verbosity: u8,
    ) -> JoinHandle<()> {
        let runner = ClientConnection {
            id,
            connection,
            verbosity,
            state: State::Uninitialized(UninitializedState::new(authenticator)),
            language: None,
        };
        tokio::spawn(async move { runner.run().await })
    }

    async fn run(mut self) {
        debug!("starting Luanti server runner");
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
            // TODO(kawogi) review whether this select remains useful after completion of the server's base implementation
            tokio::select! {
                message = self.connection.recv() => {
                    trace!("conn.recv: {message:?}");
                    let message = message?;
                    self.maybe_show(&message);
                    self.handle_client_message(message).await?;
                },
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
                    self.state = State::Authenticating(state.next());
                } else {
                    debug!("initialization is still incomplete");
                }
            }
            State::Authenticating(state) => {
                if state.handle_message(message, &self.connection)? {
                    debug!("authentication successfully completed; switching to setup mode");
                    self.state = State::Setup(AuthenticatingState::next());
                } else {
                    debug!("authentication is still incomplete");
                }
            }
            State::Setup(state) => {
                if state.handle_message(message) {
                    debug!("setup successfully completed; switching to loading mode");
                    let next_state = state.next();
                    self.language = next_state.language().cloned();
                    self.state = State::Loading(next_state);

                    let State::Loading(loading_state) = &mut self.state else {
                        // this constructions ensures that  `self.state` is up to date _before_
                        // sending out all media to the client
                        unreachable!();
                    };
                    loading_state.send_data(&self.connection)?;
                } else {
                    debug!("setup is still incomplete");
                }
            }
            State::Loading(_) => {
                if LoadingState::handle_message(message, &self.connection)? {
                    debug!("loading successfully completed; switching to authenticated mode");
                    self.state = State::Running(LoadingState::next());
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
}

enum State<Auth: Authenticator> {
    Uninitialized(UninitializedState<Auth>),
    Authenticating(AuthenticatingState),
    Setup(SetupState),
    Loading(LoadingState),
    Running(RunningState),
}
