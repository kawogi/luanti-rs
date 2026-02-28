use crate::authentication::Authenticator;
use crate::authentication::SrpUserAuthData;
use anyhow::Result;
use anyhow::bail;
use log::debug;
use log::info;
use log::warn;
use luanti_protocol::LuantiConnection;
use luanti_protocol::commands::CommandProperties;
use luanti_protocol::commands::client_to_server::InitSpec;
use luanti_protocol::commands::client_to_server::ToServerCommand;
use luanti_protocol::commands::server_to_client::HelloSpec;
use luanti_protocol::types::AuthMechsBitset;
use luanti_protocol::wire::packet::LATEST_PROTOCOL_VERSION;
use luanti_protocol::wire::packet::SER_FMT_VER_HIGHEST_WRITE;
use std::ops::RangeInclusive;

use super::authenticating::AuthenticatingState;

const SUPPORTED_PROTOCOL_VERSIONS: RangeInclusive<u16> =
    LATEST_PROTOCOL_VERSION..=LATEST_PROTOCOL_VERSION;
const SUPPORTED_SERIALIZATION_VERSIONS: RangeInclusive<u8> =
    SER_FMT_VER_HIGHEST_WRITE..=SER_FMT_VER_HIGHEST_WRITE;

/// The initial state after establishing the connection before any kind of communication happened.
/// The player/user name is not yet known and we're waiting for an Init-command to arrive.
pub(super) struct UninitializedState<Auth: Authenticator> {
    authenticator: Auth,
    /// upon receiving the user name the authenticator will be used to retrieve the user's
    /// authentication data.
    user_auth_data: Option<SrpUserAuthData>,
}

impl<Auth: Authenticator + 'static> UninitializedState<Auth> {
    #[must_use]
    pub(super) fn new(authenticator: Auth) -> Self {
        Self {
            authenticator,
            user_auth_data: None,
        }
    }

    /// This handles the first message a client sends after a connect
    pub(crate) async fn handle_message(
        &mut self,
        message: ToServerCommand,
        connection: &LuantiConnection,
    ) -> Result<bool> {
        let init_spec = match message {
            ToServerCommand::Init(init_spec) => init_spec,
            unexpected => {
                warn!(
                    "uninitialized: ignoring unexpected client message: {message_name}",
                    message_name = unexpected.command_name()
                );
                return Ok(false);
            }
        };

        let InitSpec {
            serialization_ver_max,
            supp_compr_modes: _unused,
            min_net_proto_version,
            max_net_proto_version,
            user_name,
        } = *init_spec;

        info!("New player tries to connect: {user_name}");
        debug!("Client max serialization version: {serialization_ver_max}");
        debug!("Client protocol versions: {min_net_proto_version}..{max_net_proto_version}");

        let protocol_version = {
            // intersect version ranges
            let min_version = (*SUPPORTED_PROTOCOL_VERSIONS.start()).max(min_net_proto_version);
            let max_version = (*SUPPORTED_PROTOCOL_VERSIONS.end()).min(max_net_proto_version);
            if min_version > max_version {
                bail!(
                    "unsupported protocol version. Only {min}..{max} is supported, but {min_net_proto_version}..{max_net_proto_version} was requested",
                    min = SUPPORTED_PROTOCOL_VERSIONS.start(),
                    max = SUPPORTED_PROTOCOL_VERSIONS.end(),
                );
            }
            max_version
        };
        debug!("negotiated protocol version {protocol_version}");

        let serialization_version = {
            // intersect version ranges
            let min_version = *SUPPORTED_SERIALIZATION_VERSIONS.start();
            let max_version = (*SUPPORTED_SERIALIZATION_VERSIONS.end()).min(serialization_ver_max);
            if min_version > max_version {
                bail!(
                    "unsupported serialization version. Only {min}..{max} is supported, but 0..{max_net_proto_version} was requested",
                    min = SUPPORTED_PROTOCOL_VERSIONS.start(),
                    max = SUPPORTED_PROTOCOL_VERSIONS.end(),
                );
            }
            max_version
        };
        debug!("negotiated serialization_version version {serialization_version}");

        // TODO(kawogi) Verify that the technical protocol switch is transparently performed by the underlying connection implementation

        assert!(
            self.user_auth_data
                .replace(self.authenticator.load(user_name).await?)
                .is_none(),
            "user's auth data has already been loaded"
        );

        connection.send(HelloSpec {
            serialization_version,
            compression_mode: 0, // unused field
            protocol_version,
            // TODO(kawogi) align those flags with the registered auth-providers (or transparently proxy them into a secure authentication or just don't implement all of them)
            auth_mechs: AuthMechsBitset::default(),
            username_legacy: String::new(), // always empty
        })?;

        Ok(true)
    }

    pub(crate) fn next(&mut self) -> AuthenticatingState {
        AuthenticatingState::new(
            self.user_auth_data
                .take()
                .expect("tried to progress to the next state which isn't available (this is either a double-call or a premature one)"),
        )
    }
}
