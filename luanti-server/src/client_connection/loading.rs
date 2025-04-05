use anyhow::Result;
use log::info;
use log::warn;
use luanti_protocol::LuantiConnection;
use luanti_protocol::commands::CommandProperties;
use luanti_protocol::commands::client_to_server::ClientReadySpec;
use luanti_protocol::commands::client_to_server::ToServerCommand;
use luanti_protocol::commands::server_to_client::AnnounceMediaSpec;
use luanti_protocol::commands::server_to_client::ItemdefCommand;
use luanti_protocol::commands::server_to_client::ItemdefList;
use luanti_protocol::commands::server_to_client::NodedefSpec;
use luanti_protocol::types::NodeDefManager;

use super::RunningState;

/// The state after a successful setup.
/// In this state all map data, media, etc. will be submitted
pub(super) struct LoadingState {
    language: Option<String>,
}

impl LoadingState {
    #[must_use]
    pub(super) fn new(language: Option<String>) -> Self {
        Self { language }
    }

    pub(super) fn send_data(&self, connection: &LuantiConnection) -> Result<()> {
        #[expect(
            unused_variables,
            reason = "// TODO(kawogi) This might come in handy for loading the resources"
        )]
        let language = self.language.as_ref();

        let itemdef_list = ItemdefList {
            itemdef_manager_version: 0,
            defs: vec![],
            aliases: vec![],
        };

        let node_def_manager = NodeDefManager {
            content_features: vec![],
        };

        connection.send(ItemdefCommand {
            item_def: itemdef_list,
        })?;

        connection.send(NodedefSpec {
            node_def: node_def_manager,
        })?;

        connection.send(AnnounceMediaSpec {
            files: vec![],
            remote_servers: String::new(),
        })?;

        Ok(())
    }

    pub(crate) fn handle_message(message: ToServerCommand) -> bool {
        let client_ready_spec = match message {
            ToServerCommand::ClientReady(client_ready_spec) => client_ready_spec,
            unexpected => {
                warn!(
                    "ignoring received unexpected client message: {message_name}",
                    message_name = unexpected.command_name()
                );
                return false;
            }
        };

        let ClientReadySpec {
            major_ver: _,
            minor_ver: _,
            patch_ver: _,
            reserved: _,
            full_ver,
            formspec_ver,
        } = *client_ready_spec;

        info!(
            "Client ready: v{full_ver}, formspec v{}",
            formspec_ver
                .as_ref()
                .map_or("<none>".into(), ToString::to_string)
        );

        true
    }

    pub(crate) fn next() -> RunningState {
        RunningState::new()
    }

    pub(crate) fn language(&self) -> Option<&String> {
        self.language.as_ref()
    }
}
