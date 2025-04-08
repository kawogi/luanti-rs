use std::vec;

use anyhow::Result;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use glam::I16Vec3;
use log::debug;
use log::info;
use log::warn;
use luanti_protocol::LuantiConnection;
use luanti_protocol::commands::CommandProperties;
use luanti_protocol::commands::client_to_server::ClientReadySpec;
use luanti_protocol::commands::client_to_server::RequestMediaSpec;
use luanti_protocol::commands::client_to_server::ToServerCommand;
use luanti_protocol::commands::server_to_client::AnnounceMediaSpec;
use luanti_protocol::commands::server_to_client::BlockdataSpec;
use luanti_protocol::commands::server_to_client::ItemdefCommand;
use luanti_protocol::commands::server_to_client::ItemdefList;
use luanti_protocol::commands::server_to_client::MediaSpec;
use luanti_protocol::commands::server_to_client::NodedefSpec;
use luanti_protocol::types::AlignStyle;
use luanti_protocol::types::ContentFeatures;
use luanti_protocol::types::DrawType;
use luanti_protocol::types::MapBlock;
use luanti_protocol::types::MapNode;
use luanti_protocol::types::MapNodesBulk;
use luanti_protocol::types::MediaAnnouncement;
use luanti_protocol::types::MediaFileData;
use luanti_protocol::types::NodeBox;
use luanti_protocol::types::NodeDefManager;
use luanti_protocol::types::NodeMetadataList;
use luanti_protocol::types::SColor;
use luanti_protocol::types::SimpleSoundSpec;
use luanti_protocol::types::TileAnimationParams;
use luanti_protocol::types::TileDef;
use sha1::Digest;

use super::RunningState;

const CONTENT_FEATURES_VERSION: u8 = 13;

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

    #[expect(
        clippy::too_many_lines,
        reason = "// TODO extract creation of demo content"
    )]
    pub(super) fn send_data(&self, connection: &LuantiConnection) -> Result<()> {
        #![expect(clippy::similar_names, reason = "English being English")]

        #[expect(
            unused_variables,
            reason = "// TODO(kawogi) This might come in handy for loading the resources"
        )]
        let language = self.language.as_ref();

        let tiledef = TileDef {
            name: "rust_tile_32.png".into(),
            animation: TileAnimationParams::None,
            backface_culling: true,
            tileable_horizontal: false,
            tileable_vertical: false,
            color_rgb: Some(SColor::RED.rgb().into()),
            scale: 0,
            align_style: AlignStyle::Node,
        };

        let tiledef_overlay = TileDef {
            name: String::new(),
            animation: TileAnimationParams::None,
            backface_culling: true,
            tileable_horizontal: false,
            tileable_vertical: false,
            color_rgb: None,
            scale: 0,
            align_style: AlignStyle::Node,
        };

        let tiledef_special = TileDef {
            name: String::new(),
            animation: TileAnimationParams::None,
            backface_culling: true,
            tileable_horizontal: false,
            tileable_vertical: false,
            color_rgb: None,
            scale: 0,
            align_style: AlignStyle::Node,
        };

        let sound_footstep = SimpleSoundSpec {
            name: String::new(),
            gain: 1.0,
            pitch: 1.0,
            fade: 1.0,
        };

        let sound_dig = SimpleSoundSpec {
            name: String::new(),
            gain: 1.0,
            pitch: 1.0,
            fade: 1.0,
        };

        let sound_dug = SimpleSoundSpec {
            name: String::new(),
            gain: 1.0,
            pitch: 1.0,
            fade: 1.0,
        };

        let content_feature = ContentFeatures {
            version: CONTENT_FEATURES_VERSION,
            name: "block_of_rust".into(),
            groups: Vec::new(),
            param_type: 0,
            param_type_2: 0,
            drawtype: DrawType::Normal,
            mesh: String::new(),
            visual_scale: 1.0,
            unused_six: 6,
            tiledef: [
                tiledef.clone(),
                tiledef.clone(),
                tiledef.clone(),
                tiledef.clone(),
                tiledef.clone(),
                tiledef.clone(),
            ],
            tiledef_overlay: [
                tiledef_overlay.clone(),
                tiledef_overlay.clone(),
                tiledef_overlay.clone(),
                tiledef_overlay.clone(),
                tiledef_overlay.clone(),
                tiledef_overlay.clone(),
            ],
            tiledef_special: vec![
                tiledef_special.clone(),
                tiledef_special.clone(),
                tiledef_special.clone(),
                tiledef_special.clone(),
                tiledef_special.clone(),
                tiledef_special.clone(),
            ],
            alpha_for_legacy: 255,
            red: 0,
            green: 255,
            blue: 0,
            palette_name: String::new(),
            waving: 0,
            connect_sides: 0,
            connects_to_ids: Vec::new(),
            post_effect_color: SColor::new(0, 0, 255, 255),
            leveled: 0,
            light_propagates: 0,
            sunlight_propagates: 0,
            light_source: 0,
            is_ground_content: true,
            walkable: true,
            pointable: true,
            diggable: true,
            climbable: false,
            buildable_to: true,
            rightclickable: true,
            damage_per_second: 0,
            liquid_type_bc: 0,
            liquid_alternative_flowing: String::new(),
            liquid_alternative_source: String::new(),
            liquid_viscosity: 0,
            liquid_renewable: false,
            liquid_range: 0,
            drowning: 0,
            floodable: false,
            node_box: NodeBox::Regular,
            selection_box: NodeBox::Regular,
            collision_box: NodeBox::Regular,
            sound_footstep,
            sound_dig,
            sound_dug,
            legacy_facedir_simple: false,
            legacy_wallmounted: false,
            node_dig_prediction: None,
            leveled_max: None,
            alpha: None,
            move_resistance: None,
            liquid_move_physics: None,
        };

        let png = include_bytes!("../../assets/rust_tile_32.png");
        let mut hasher = sha1::Sha1::new();
        hasher.update(png);
        let hash = hasher.finalize();
        let sha1_base64 = STANDARD.encode(hash);

        let media_announcement = MediaAnnouncement {
            name: "rust_tile_32.png".into(),
            sha1_base64,
        };

        let itemdef_list = ItemdefList {
            itemdef_manager_version: 0,
            defs: vec![],
            aliases: vec![],
        };

        let node_def_manager = NodeDefManager {
            content_features: vec![(10, content_feature)],
        };

        connection.send(ItemdefCommand {
            item_def: itemdef_list,
        })?;

        connection.send(NodedefSpec {
            node_def: node_def_manager,
        })?;

        connection.send(AnnounceMediaSpec {
            files: vec![media_announcement],
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

        let nodes = std::array::from_fn(|_index| MapNode {
            param0: 10,
            param1: 0,
            param2: 0,
        });

        let block = MapBlock {
            is_underground: true,
            day_night_diff: true,
            generated: true,
            lighting_complete: Some(0),
            nodes: MapNodesBulk { nodes },
            node_metadata: NodeMetadataList { metadata: vec![] },
        };

        let blockdata = BlockdataSpec {
            pos: I16Vec3::new(0, -1, 0),
            block,
            network_specific_version: 0,
        };

        connection.send(blockdata)?;

        Ok(true)
    }

    fn handle_request_media(
        request_media_spec: RequestMediaSpec,
        connection: &LuantiConnection,
    ) -> Result<bool> {
        let RequestMediaSpec { files } = request_media_spec;

        debug!("client requested files: {files:?}");
        for file in files {
            debug!("sending file: {file}");

            let media_file_data = MediaFileData {
                name: file,
                data: include_bytes!("../../assets/rust_tile_32.png").to_vec(),
            };

            connection.send(MediaSpec {
                num_bunches: 1,
                bunch_index: 0,
                files: vec![media_file_data],
            })?;
        }

        Ok(false)
    }

    pub(crate) fn next() -> RunningState {
        RunningState::new()
    }

    pub(crate) fn language(&self) -> Option<&String> {
        self.language.as_ref()
    }
}
