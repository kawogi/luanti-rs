//! Contains an implementation of an authenticator which permits access for all users with all
//! passwords.

use std::pin::Pin;

use anyhow::Result;
use rand::Rng;

use super::{Authenticator, SrpUserAuthData};

/// Implements an authenticator which permits access to every user with every password.
/// This is meant to be used for testing or in environments where protection is achieved by other
/// means or isn't necessary at all.
#[derive(Clone)]
pub struct DummyAuthenticator;

impl Authenticator for DummyAuthenticator {
    fn load(
        &self,
        user_name: String,
    ) -> Pin<Box<dyn Future<Output = Result<SrpUserAuthData>> + Send + '_>> {
        let mut salt = [0_u8; 64];
        rand::rng().fill_bytes(&mut salt);
        let mut verifier = [0_u8; 64];
        rand::rng().fill_bytes(&mut verifier);

        let name = user_name.to_lowercase();
        Box::pin(std::future::ready(Ok(SrpUserAuthData {
            name: name.to_ascii_lowercase(),
            display_name: name,
            salt: salt.to_vec(),
            verifier: verifier.to_vec(),
        })))
    }
}
