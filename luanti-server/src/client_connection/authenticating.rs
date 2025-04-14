use crate::authentication::SrpUserAuthData;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use glam::Vec3;
use log::info;
use log::warn;
use luanti_protocol::LuantiConnection;
use luanti_protocol::commands::CommandProperties;
use luanti_protocol::commands::client_to_server::ToServerCommand;
use luanti_protocol::commands::{
    client_to_server::{SrpBytesASpec, SrpBytesMSpec},
    server_to_client::{AuthAcceptSpec, SrpBytesSBSpec},
};
use rand::RngCore;
use sha2::Sha256;
use srp::{
    groups::G_2048,
    server::{SrpServer, SrpServerVerifier},
};

use super::SetupState;

type Verifier = SrpServerVerifier<Sha256>;

/// The state of a connection after receiving the player's name.
pub(super) struct AuthenticatingState {
    user_auth_data: SrpUserAuthData,
    state: SrpAuthState,
}

impl AuthenticatingState {
    #[must_use]
    pub(super) fn new(user_auth_data: SrpUserAuthData) -> Self {
        Self {
            user_auth_data,
            state: SrpAuthState::Uninitialized,
        }
    }

    /// Changes the internal state of this authentication based on the received client message.
    pub(super) fn handle_message(
        &mut self,
        message: ToServerCommand,
        connection: &LuantiConnection,
    ) -> Result<bool> {
        match (&mut self.state, message) {
            // a `BytesA`-messages performs a state transition `Init` → `Init2`
            (SrpAuthState::Uninitialized, ToServerCommand::SrpBytesA(srp_bytes_a)) => {
                if let Some(verifier) =
                    Self::handle_srp_bytes_a(&self.user_auth_data, *srp_bytes_a, connection)?
                {
                    self.state = SrpAuthState::Verification { verifier };
                }
                Ok(false)
            }
            // a `BytesM`-messages performs a state transition `Init2` → `Authenticated`
            (
                SrpAuthState::Verification { verifier },
                ToServerCommand::SrpBytesM(srp_bytes_mspec),
            ) => {
                if Self::handle_srp_bytes_mspec(
                    &self.user_auth_data,
                    verifier,
                    *srp_bytes_mspec,
                    connection,
                )? {
                    self.state = SrpAuthState::Authenticated;
                    Ok(true)
                } else {
                    Ok(false)
                }
            }
            // we received a message which was unexpected for the current state
            (state, unexpected) => {
                warn!(
                    "ignoring unexpected message for state {state}: {message}",
                    state = state.as_str(),
                    message = unexpected.command_name()
                );
                Ok(false)
            }
        }
    }

    fn handle_srp_bytes_a(
        user_data: &SrpUserAuthData,
        srp_bytes_a: SrpBytesASpec,
        conn: &LuantiConnection,
    ) -> Result<Option<Verifier>> {
        // the client sends `A` earlier than usual because the required `g` is well-known
        // (pre-shared) and doesn't need to be sent by the server
        let SrpBytesASpec {
            bytes_a: srp_public_a,
            based_on,
        } = srp_bytes_a;

        if based_on == 0 {
            // TODO(kawogi) respond with a proper auth rejection to the client
            // TODO(kawogi) maybe remove this specific test in favor of a general incompatibility if modern clients (based on their protocol version) prove to be unable to ever send this value
            bail!("server doesn't support legacy authentication");
        } else if based_on > 1 {
            // TODO(kawogi) respond with a proper auth rejection to the client
            bail!("server doesn't support `based_on={based_on}` for SRP authentication");
        }

        let srp_server = SrpServer::<Sha256>::new(&G_2048);

        let mut srp_private_b = [0_u8; 256];
        rand::rng().fill_bytes(&mut srp_private_b);
        let srp_b_pub = srp_server.compute_public_ephemeral(&srp_private_b, &user_data.verifier);

        let verifier = srp_server
            .process_reply(&srp_private_b, &user_data.verifier, &srp_public_a)
            .map_err(|error| anyhow!("{error}"))?;

        let srp_bytes_b = SrpBytesSBSpec {
            s: user_data.salt.clone(),
            b: srp_b_pub,
        };
        conn.send(srp_bytes_b)?;

        Ok(Some(verifier))
    }

    fn handle_srp_bytes_mspec(
        user_data: &SrpUserAuthData,
        verifier: &Verifier,
        srp_bytes_mspec: SrpBytesMSpec,
        connection: &LuantiConnection,
    ) -> Result<bool> {
        let SrpBytesMSpec { bytes_m } = srp_bytes_mspec;

        match verifier.verify_client(&bytes_m) {
            Ok(()) => {
                info!(
                    "player '{name}' was successfully authenticated",
                    name = user_data.display_name
                );
            }
            Err(error) => {
                warn!(
                    "failed to authenticate player '{name}': {error}",
                    name = user_data.display_name,
                );
                // FIXME(kawogi) remove this once there's a proper implementation in place
                warn!("MOCK-AUTHENTICATOR GRANTS ACCESS NONETHELESS!");
            }
        }

        let auth_accept = AuthAcceptSpec {
            // TODO(kawogi) load from saved game or default spawn position
            player_pos: Vec3 {
                x: 0.0 * 10.0,
                y: (0.0 + 0.5) * 10.0,
                z: 0.0 * 10.0,
            },
            // TODO(kawogi) load from actual map?
            map_seed: 0,
            // TODO(kawogi) what is this value?
            recommended_send_interval: 0.05,
            // TODO(kawogi) what is this value? look up `choseAuthMech` in original source code
            sudo_auth_methods: 2,
        };
        connection.send(auth_accept)?;

        Ok(true)
    }

    pub(crate) fn next() -> SetupState {
        SetupState::new()
    }
}

/// Remembers the current state of an SRP authentication handshake.
#[derive(Default)]
enum SrpAuthState {
    /// The initial state after creation, ready for authenticating the given user.
    /// Now waiting for `BytesA`.
    #[default]
    Uninitialized,
    /// The state after receiving `BytesA` from the client.
    /// Now waiting for `BytesM`.
    Verification { verifier: Verifier },
    /// The state after receiving `BytesM` from the client.
    /// The user is now fully authenticated.
    Authenticated,
}

impl SrpAuthState {
    /// internal helper method to return the name of the current state
    fn as_str(&self) -> &'static str {
        match self {
            SrpAuthState::Uninitialized => "uninitialized",
            SrpAuthState::Verification { .. } => "verification",
            SrpAuthState::Authenticated => "authenticated",
        }
    }
}
