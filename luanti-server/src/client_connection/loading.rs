use std::{array, vec};

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
    types::{
        AlignStyle, ContentFeatures, DrawType, MediaAnnouncement, MediaFileData, NodeBox,
        NodeDefManager, SColor, SimpleSoundSpec, TileAnimationParams, TileDef,
    },
};
use sha1::Digest;

const CONTENT_FEATURES_VERSION: u8 = 13;

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

    #[expect(
        clippy::too_many_lines,
        reason = "// TODO extract creation of demo content"
    )]
    pub(super) fn send_data(&self, connection: &LuantiConnection) -> Result<()> {
        #[expect(
            unused_variables,
            reason = "// TODO(kawogi) This might come in handy for loading the resources"
        )]
        let language = self.language.as_ref();

        let tile_dirt = tile_def("demo_dirt.png");
        let tile_grass_east = tile_def("demo_grass_east.png");
        let tile_grass_north = tile_def("demo_grass_north.png");
        let tile_grass_south = tile_def("demo_grass_south.png");
        let tile_grass_west = tile_def("demo_grass_west.png");
        let tile_grass = tile_def("demo_grass.png");
        let tile_sand = tile_def("demo_sand.png");
        let tile_stone = tile_def("demo_stone.png");
        let tile_water = tile_def("demo_water.png");

        let tile_none = TileDef {
            name: String::new(),
            animation: TileAnimationParams::None,
            backface_culling: true,
            tileable_horizontal: false,
            tileable_vertical: false,
            color_rgb: None,
            scale: 0,
            align_style: AlignStyle::Node,
        };

        let content_dirt = content_features("basenodes:dirt", &[&tile_dirt], &[]);
        let content_dirt_with_grass = content_features(
            "basenodes:dirt_with_grass",
            &[&tile_dirt],
            &[
                &tile_grass,
                &tile_none,
                &tile_grass_east,
                &tile_grass_north,
                &tile_grass_south,
                &tile_grass_west,
            ],
        );
        let content_sand = content_features("basenodes:sand", &[&tile_sand], &[]);
        let content_stone = content_features("basenodes:stone", &[&tile_stone], &[]);
        let content_water_source = content_features("basenodes:water_source", &[&tile_water], &[]);
        let content_water_flowing =
            content_features("basenodes:water_flowing", &[&tile_water], &[]);

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

        let node_def_manager = NodeDefManager {
            content_features: vec![
                (1, content_stone),
                (2, content_sand),
                (3, content_dirt_with_grass),
                (4, content_dirt),
                (5, content_water_source),
                (6, content_water_flowing),
            ],
        };

        connection.send(ItemdefCommand {
            item_def: itemdef_list,
        })?;

        connection.send(NodedefSpec {
            node_def: node_def_manager,
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

// #[expect(clippy::too_many_lines, reason = "//TODO fix this later")]
#[expect(clippy::similar_names, reason = "English being English")]
fn content_features(name: &str, tiles: &[&TileDef], overlays: &[&TileDef]) -> ContentFeatures {
    let tile_none = TileDef {
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

    let tiledef =
        array::from_fn(|index| (*tiles.get(index).or(tiles.last()).unwrap_or(&&tile_none)).clone());
    let tiledef_overlay = array::from_fn(|index| {
        (*overlays
            .get(index)
            .or(overlays.last())
            .unwrap_or(&&tile_none))
        .clone()
    });

    ContentFeatures {
        version: CONTENT_FEATURES_VERSION,
        name: name.into(),
        groups: Vec::new(),
        param_type: 0,
        param_type_2: 0,
        drawtype: DrawType::Normal,
        mesh: String::new(),
        visual_scale: 1.0,
        unused_six: 6,
        tiledef,
        tiledef_overlay,
        tiledef_special: vec![
            tile_none.clone(),
            tile_none.clone(),
            tile_none.clone(),
            tile_none.clone(),
            tile_none.clone(),
            tile_none.clone(),
        ],
        alpha_for_legacy: 255,
        red: 255,
        green: 255,
        blue: 255,
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
    }
}

fn tile_def(name: &str) -> TileDef {
    TileDef {
        name: name.into(),
        animation: TileAnimationParams::None,
        backface_culling: true,
        tileable_horizontal: false,
        tileable_vertical: false,
        color_rgb: None,
        scale: 0,
        align_style: AlignStyle::Node,
    }
}
