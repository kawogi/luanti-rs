//! Contains the implementation for authenticating a user.

pub mod dummy;

use anyhow::{Result, anyhow, bail};
use log::{info, warn};
use luanti_protocol::{
    LuantiConnection,
    commands::{
        CommandProperties,
        client_to_server::{SrpBytesASpec, SrpBytesMSpec, ToServerCommand},
        server_to_client::{AuthAcceptSpec, SrpBytesSBSpec},
    },
    types::v3f,
};
use rand::RngCore;
use sha2::Sha256;
use srp::{
    groups::G_2048,
    server::{SrpServer, SrpServerVerifier},
};
use std::pin::Pin;

/// An `Authenticator` provides the server with the information necessary to authenticate a single
/// user.
pub trait Authenticator: Send + Sync + Clone {
    /// Tries to create a new authenticator for a named user.
    ///
    /// An implementation shall try to look up this user and fetch the associated security tokens.
    ///
    /// # Errors
    ///
    /// Returns an error if the user name wasn't found or was otherwise invalid.
    fn load(
        &self,
        user_name: String,
    ) -> Pin<Box<dyn Future<Output = Result<SrpUserAuthData>> + Send + '_>>;
}

/// Contains all information the SRC authentication mechanism needs to authenticate a user.
pub struct SrpUserAuthData {
    /// The (non-technical) name that has been provided by the user. This might contain special
    /// characters or have mixed casing. This may be used as display name.
    pub display_name: String,
    /// Technical name (key) of the user's record. This is usually a normalized version of the
    /// `display_name`
    pub name: String,
    /// The salt value (`s`) that has been provided during the creation of the user.
    /// This is required by the SRP-authentication mechanism.
    pub salt: Vec<u8>,
    /// The verifier value (`v`) that has been provided during the creation of the user.
    /// This is required by the SRP-authentication mechanism.
    pub verifier: Vec<u8>,
}

impl SrpUserAuthData {}

/// Remembers the current state of an SRP authentication handshake.
#[derive(Default)]
pub(crate) enum SrpAuthState {
    /// The initial state after creation.
    /// Now waiting for the user data.
    ///
    /// This is also being used internally as temporary value during state transitions
    #[default]
    None,
    /// The state after receiving the information about the user to authenticate.
    /// Now waiting for `BytesA`.
    Init { user_data: SrpUserAuthData },
    /// The state after receiving `BytesA` from the client.
    /// Now waiting for `BytesM`.
    Init2 {
        user_data: SrpUserAuthData,
        verifier: SrpServerVerifier<Sha256>,
    },
    Authenticated {
        /// The (non-technical) name that has been provided by the user. This might contain special
        /// characters or have mixed casing. This may be used as display name.
        display_name: String,
        /// Technical name (key) of the user's record. This is usually a normalized version of the
        /// `display_name`
        name: String,
    },
}

impl SrpAuthState {
    /// Create an initialized state, ready for authenticating the given user.
    pub(crate) fn new(user_data: SrpUserAuthData) -> Self {
        Self::Init { user_data }
    }

    pub(crate) fn handle_client_message(
        self,
        message: ToServerCommand,
        conn: &LuantiConnection,
    ) -> Result<Self> {
        let result = match (self, message) {
            (SrpAuthState::Init { user_data }, ToServerCommand::SrpBytesA(srp_bytes_a)) => {
                Self::handle_srp_bytes_a(user_data, *srp_bytes_a, conn)?
            }
            (
                SrpAuthState::Init2 {
                    user_data,
                    verifier,
                },
                ToServerCommand::SrpBytesM(srp_bytes_mspec),
            ) => Self::handle_srp_bytes_mspec(user_data, &verifier, *srp_bytes_mspec, conn)?,
            (state, unexpected) => {
                warn!(
                    "unexpected message for state {state}: {message}",
                    state = state.as_str(),
                    message = unexpected.command_name()
                );
                // keep the old state
                state
            }
        };

        Ok(result)
    }

    fn handle_srp_bytes_a(
        user_data: SrpUserAuthData,
        srp_bytes_a: SrpBytesASpec,
        conn: &LuantiConnection,
    ) -> Result<Self> {
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

        // let (salt, v) = get_user(&username);
        // TODO(kawogi) look up salt and verifier for this user
        // let mut srp_user_salt = [0_u8; 64];
        // rand::rng().fill_bytes(&mut srp_user_salt);
        // let mut srp_user_verifier = [0_u8; 64];
        // rand::rng().fill_bytes(&mut srp_user_verifier);

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

        Ok(Self::Init2 {
            user_data,
            verifier,
        })
    }

    fn handle_srp_bytes_mspec(
        user_data: SrpUserAuthData,
        verifier: &SrpServerVerifier<Sha256>,
        srp_bytes_mspec: SrpBytesMSpec,
        conn: &LuantiConnection,
    ) -> Result<Self> {
        let SrpBytesMSpec { bytes_m } = srp_bytes_mspec;

        match verifier.verify_client(&bytes_m) {
            Ok(()) => {
                info!(
                    "player '{player_name}' was sucessfully authenticated",
                    player_name = user_data.display_name
                );
            }
            Err(error) => {
                warn!(
                    "failed to authenticate player '{player_name}': {error}",
                    player_name = user_data.display_name,
                );
                // FIXME(kawogi) remove this once there's a proper implementation in place
                warn!("MOCK-AUTHENTICATOR GRANTS ACCESS NONETHELESS!");
            }
        }

        let auth_accept = AuthAcceptSpec {
            player_pos: v3f {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            // TODO(kawogi) load from actual map?
            map_seed: 0,
            // TODO(kawogi) what is this value?
            recommended_send_interval: 0.05,
            // TODO(kawogi) what is this value?
            sudo_auth_methods: 2,
        };
        conn.send(auth_accept)?;

        Ok(Self::Authenticated {
            display_name: user_data.display_name,
            name: user_data.name,
        })
    }

    /// internal helper method to return the name of the current state
    fn as_str(&self) -> &'static str {
        match self {
            SrpAuthState::None => "none",
            SrpAuthState::Init { .. } => "init",
            SrpAuthState::Init2 { .. } => "init2",
            SrpAuthState::Authenticated { .. } => "authenticated",
        }
    }
}
