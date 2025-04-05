//! Contains the implementation for authenticating a user.

pub mod dummy;

use anyhow::Result;
use std::pin::Pin;

/// An `Authenticator` provides the server with the information necessary to authenticate a single
/// user via SRP.
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

/// Contains all information the SRP authentication mechanism needs to authenticate a user.
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
