use std::{sync::Arc, vec};

use crate::MediaRegistry;
use anyhow::Result;
use log::{debug, error, info, warn};
use luanti_protocol::{
    LuantiConnection,
    commands::{
        CommandProperties,
        client_to_server::{ClientReadySpec, RequestMediaSpec, ToServerCommand},
        server_to_client::{
            AnnounceMediaSpec, ItemdefCommand, ItemdefList, MediaSpec, NodedefSpec, PrivilegesSpec,
        },
    },
    types::{MediaAnnouncement, MediaFileData, NodeDefManager},
};

/// The state after a successful setup.
/// In this state all map data, media, etc. will be submitted
pub(super) struct LoadingState {
    language: Option<String>,
    media: Arc<MediaRegistry>,
    // pub(crate) player_key: SharedStr,
}

impl LoadingState {
    #[must_use]
    pub(super) fn new(language: Option<String>, media: Arc<MediaRegistry>) -> Self {
        Self {
            language,
            media,
            // player_key,
        }
    }

    pub(super) fn send_data(
        &self,
        connection: &LuantiConnection,
        node_def: &NodeDefManager,
        media: &MediaRegistry,
    ) -> Result<()> {
        #[expect(
            unused_variables,
            reason = "// TODO(kawogi) This might come in handy for loading the resources"
        )]
        let language = self.language.as_ref();

        let files = media
            .hashes()
            .map(|(name, sha1_base64)| MediaAnnouncement {
                name: name.to_string(),
                sha1_base64,
            })
            .collect();

        let itemdef_list = ItemdefList {
            itemdef_manager_version: 0,
            defs: vec![],
            aliases: vec![],
        };

        connection.send(ItemdefCommand {
            item_def: itemdef_list,
        })?;

        connection.send(NodedefSpec {
            node_def: node_def.clone(),
        })?;

        connection.send(AnnounceMediaSpec {
            files,
            remote_servers: String::new(),
        })?;

        Ok(())
    }

    pub(crate) fn handle_message(
        &self,
        message: ToServerCommand,
        connection: &LuantiConnection,
    ) -> Result<bool> {
        match message {
            ToServerCommand::ClientReady(client_ready_spec) => {
                Self::handle_client_ready(*client_ready_spec, connection)
            }
            ToServerCommand::RequestMedia(request_media_spec) => {
                self.handle_request_media(*request_media_spec, connection)
            }
            unexpected => {
                warn!(
                    "loading: ignoring unexpected client message: {message_name}",
                    message_name = unexpected.command_name()
                );
                Ok(false)
            }
        }
    }

    fn handle_client_ready(
        client_ready_spec: ClientReadySpec,
        connection: &LuantiConnection,
    ) -> Result<bool> {
        let ClientReadySpec {
            major_ver: _,
            minor_ver: _,
            patch_ver: _,
            reserved: _,
            full_ver,
            formspec_ver,
        } = client_ready_spec;

        info!(
            "Client ready: v{full_ver}, formspec v{}",
            formspec_ver
                .as_ref()
                .map_or("<none>".into(), ToString::to_string)
        );

        connection.send(PrivilegesSpec {
            privileges: vec![
                "fly".into(),
                "fast".into(),
                "noclip".into(),
                "rollback".into(),
                "debug".into(),
            ],
        })?;

        Ok(true)
    }

    fn handle_request_media(
        &self,
        request_media_spec: RequestMediaSpec,
        connection: &LuantiConnection,
    ) -> Result<bool> {
        let RequestMediaSpec { files } = request_media_spec;

        debug!("client requested files: {files:?}");
        let mut media_file_data = vec![];
        for file in files {
            debug!("sending file: {file}");

            let Some(data) = self.media.file_content(&file)? else {
                error!("could not find file: {file}");
                continue;
            };

            media_file_data.push(MediaFileData { name: file, data });
        }

        connection.send(MediaSpec {
            num_bunches: 1,
            bunch_index: 0,
            files: media_file_data,
        })?;

        Ok(false)
    }

    pub(crate) fn language(&self) -> Option<&String> {
        self.language.as_ref()
    }
}
