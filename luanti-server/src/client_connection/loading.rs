use std::vec;

use anyhow::Result;
use base64::{Engine, engine::general_purpose::STANDARD};
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
use sha1::Digest;

/// The state after a successful setup.
/// In this state all map data, media, etc. will be submitted
pub(super) struct LoadingState {
    language: Option<String>,
    // pub(crate) player_key: SharedStr,
}

impl LoadingState {
    #[must_use]
    pub(super) fn new(language: Option<String>) -> Self {
        Self {
            language,
            // player_key,
        }
    }

    pub(super) fn send_data(
        &self,
        connection: &LuantiConnection,
        node_def: &NodeDefManager,
    ) -> Result<()> {
        #[expect(
            unused_variables,
            reason = "// TODO(kawogi) This might come in handy for loading the resources"
        )]
        let language = self.language.as_ref();

        let textures = vec![
            (
                "demo_dirt.png",
                include_bytes!("../../assets/demo_dirt.png").as_slice(),
            ),
            (
                "demo_grass_east.png",
                include_bytes!("../../assets/demo_grass_east.png").as_slice(),
            ),
            (
                "demo_grass_north.png",
                include_bytes!("../../assets/demo_grass_north.png").as_slice(),
            ),
            (
                "demo_grass_south.png",
                include_bytes!("../../assets/demo_grass_south.png").as_slice(),
            ),
            (
                "demo_grass_west.png",
                include_bytes!("../../assets/demo_grass_west.png").as_slice(),
            ),
            (
                "demo_grass.png",
                include_bytes!("../../assets/demo_grass.png").as_slice(),
            ),
            (
                "demo_sand.png",
                include_bytes!("../../assets/demo_sand.png").as_slice(),
            ),
            (
                "demo_stone.png",
                include_bytes!("../../assets/demo_stone.png").as_slice(),
            ),
            (
                "demo_water.png",
                include_bytes!("../../assets/demo_water.png").as_slice(),
            ),
        ];

        let mut files = vec![];
        for (name, png) in textures {
            // let png = include_bytes!("../../assets/rust_tile_32.png");
            let mut hasher = sha1::Sha1::new();
            hasher.update(png);
            let hash = hasher.finalize();
            let sha1_base64 = STANDARD.encode(hash);

            let media_announcement = MediaAnnouncement {
                name: name.into(),
                sha1_base64,
            };
            files.push(media_announcement);
        }

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
        message: ToServerCommand,
        connection: &LuantiConnection,
    ) -> Result<bool> {
        match message {
            ToServerCommand::ClientReady(client_ready_spec) => {
                Self::handle_client_ready(*client_ready_spec, connection)
            }
            ToServerCommand::RequestMedia(request_media_spec) => {
                Self::handle_request_media(*request_media_spec, connection)
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
        request_media_spec: RequestMediaSpec,
        connection: &LuantiConnection,
    ) -> Result<bool> {
        let RequestMediaSpec { files } = request_media_spec;

        debug!("client requested files: {files:?}");
        let mut media_file_data = vec![];
        for file in files {
            debug!("sending file: {file}");

            let data = match file.as_str() {
                "demo_dirt.png" => include_bytes!("../../assets/demo_dirt.png").to_vec(),
                "demo_grass_east.png" => {
                    include_bytes!("../../assets/demo_grass_east.png").to_vec()
                }
                "demo_grass_north.png" => {
                    include_bytes!("../../assets/demo_grass_north.png").to_vec()
                }
                "demo_grass_south.png" => {
                    include_bytes!("../../assets/demo_grass_south.png").to_vec()
                }
                "demo_grass_west.png" => {
                    include_bytes!("../../assets/demo_grass_west.png").to_vec()
                }
                "demo_grass.png" => include_bytes!("../../assets/demo_grass.png").to_vec(),
                "demo_sand.png" => include_bytes!("../../assets/demo_sand.png").to_vec(),
                "demo_stone.png" => include_bytes!("../../assets/demo_stone.png").to_vec(),
                "demo_water.png" => include_bytes!("../../assets/demo_water.png").to_vec(),
                unknown => {
                    error!("unknown file requested: {unknown}");
                    continue;
                }
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
