use log::info;
use log::warn;
use luanti_protocol::commands::CommandProperties;
use luanti_protocol::commands::client_to_server::Init2Spec;
use luanti_protocol::commands::client_to_server::ToServerCommand;

use super::LoadingState;

/// The state after a successful authentication.
pub(super) struct SetupState {
    // player_key: SharedStr,
    language: Option<String>,
}

impl SetupState {
    #[must_use]
    pub(super) fn new() -> Self {
        Self {
            // player_key,
            language: None,
        }
    }

    pub(crate) fn handle_message(&mut self, message: ToServerCommand) -> bool {
        let init2_spec = match message {
            ToServerCommand::Init2(init2_spec) => init2_spec,
            unexpected => {
                warn!(
                    "setup: ignoring unexpected client message: {message_name}",
                    message_name = unexpected.command_name()
                );
                return false;
            }
        };

        let Init2Spec { lang } = *init2_spec;

        if let Some(language) = lang.as_ref() {
            info!("Client language: '{language}'",);
        } else {
            info!("Client language: <none>",);
        }
        self.language = lang;

        true
    }

    pub(crate) fn next(&self) -> LoadingState {
        LoadingState::new(self.language.clone())
    }
}
