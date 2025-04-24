//! Luanti server implemented in Rust
// #![expect(clippy::expect_used, reason = "//TODO improve error handling")]

#![expect(
    clippy::todo,
    clippy::expect_used,
    reason = "//TODO remove before completion of the prototype"
)]

pub mod authentication;
mod client_connection;
mod server;
mod world;

use anyhow::bail;
use authentication::dummy::DummyAuthenticator;
use clap::ArgGroup;
use clap::Parser;
use flexstr::SharedStr;
use luanti_protocol::types::AlignStyle;
use luanti_protocol::types::ContentFeatures;
use luanti_protocol::types::DrawType;
use luanti_protocol::types::NodeBox;
use luanti_protocol::types::NodeDefManager;
use luanti_protocol::types::SColor;
use luanti_protocol::types::SimpleSoundSpec;
use luanti_protocol::types::TileAnimationParams;
use luanti_protocol::types::TileDef;
use server::LuantiWorldServer;
use std::array;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use world::content_id_map::ContentIdMap;
use world::generation::flat::MapgenFlat;
use world::map_block_provider::MapBlockProvider;
use world::map_block_router::MapBlockRouter;
use world::media_registry::MediaRegistry;
use world::storage::minetestworld::MinetestworldStorage;

const CONTENT_FEATURES_VERSION: u8 = 13;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[command(group(ArgGroup::new("source").required(true).args(["listen", "bind"])))]
struct Args {
    /// Listen on port
    #[arg(group = "source", short, long)]
    listen: Option<u16>,

    /// Listen with specific bind address (ip:port)
    #[arg(group = "source", short, long)]
    bind: Option<SocketAddr>,

    /// Verbosity level (up to -vvv)
    #[arg(short, long, default_value_t = 0, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // tokio::main makes rust-analyzer fragile,
    // so put the code in a separate place.
    real_main().await
}

async fn real_main() -> anyhow::Result<()> {
    // TODO make this configurable through command line arguments
    env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .init();

    let args = Args::parse();

    let bind_addr: SocketAddr = if let Some(listen_port) = args.listen {
        // TODO(kawogi) re-enable IPv6 support
        if true {
            format!("0.0.0.0:{listen_port}").parse()?
        } else {
            format!("[::]:{listen_port}").parse()?
        }
    } else if let Some(bind_addr) = args.bind {
        bind_addr
    } else {
        bail!("One of --listen or --bind must be specified");
    };

    let mut media_registry = MediaRegistry::default();
    media_registry.load_directory("luanti-server/assets")?;

    let mut content_id_map = ContentIdMap::new();
    let content_id_stone = content_id_map.push(SharedStr::from_static("basenodes:stone"))?;
    let content_id_sand = content_id_map.push(SharedStr::from_static("basenodes:sand"))?;
    let content_id_dirt_with_grass =
        content_id_map.push(SharedStr::from_static("basenodes:dirt_with_grass"))?;
    let content_id_dirt = content_id_map.push(SharedStr::from_static("basenodes:dirt"))?;
    let content_id_water_source =
        content_id_map.push(SharedStr::from_static("basenodes:water_source"))?;
    let content_id_water_flowing =
        content_id_map.push(SharedStr::from_static("basenodes:water_flowing"))?;

    let tile_dirt = tile_def("demo_dirt.png");
    let tile_grass_east = tile_def("demo_grass_east.png");
    let tile_grass_north = tile_def("demo_grass_north.png");
    let tile_grass_south = tile_def("demo_grass_south.png");
    let tile_grass_west = tile_def("demo_grass_west.png");
    let tile_grass = tile_def("demo_grass.png");
    let tile_sand = tile_def("demo_sand.png");
    let tile_stone = tile_def("demo_stone.png");
    let tile_water = tile_def("demo_water.png");

    let tile_none = tile_def("");

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
    let content_water_flowing = content_features("basenodes:water_flowing", &[&tile_water], &[]);

    let node_def_manager = NodeDefManager {
        content_features: vec![
            (content_id_stone.0, content_stone),
            (content_id_sand.0, content_sand),
            (content_id_dirt_with_grass.0, content_dirt_with_grass),
            (content_id_dirt.0, content_dirt),
            (content_id_water_source.0, content_water_source),
            (content_id_water_flowing.0, content_water_flowing),
        ],
    };

    let world_generator = MapgenFlat;
    let storage = pollster::block_on(MinetestworldStorage::new(
        "worlds/luanti-rs",
        Arc::new(content_id_map),
    ))?;

    let (block_request_to_provider, block_request_from_router) = mpsc::unbounded_channel();
    let (block_interest_sender, block_interest_receiver) = mpsc::unbounded_channel();
    let (world_update_to_router, world_update_from_provider) = mpsc::unbounded_channel();
    let _block_provider = MapBlockProvider::new(
        block_request_from_router,
        world_update_to_router,
        Some(Box::new(storage)),
        Some(Box::new(world_generator)),
    );

    let mut server = LuantiWorldServer::new(bind_addr, args.verbose, Arc::new(node_def_manager));

    let _map_block_router = MapBlockRouter::new(
        block_request_to_provider,
        world_update_from_provider,
        block_interest_receiver,
    );

    server.start(DummyAuthenticator, block_interest_sender);
    #[expect(
        clippy::infinite_loop,
        reason = "// TODO implement a cancellation mechanism"
    )]
    loop {
        tokio::time::sleep(Duration::from_secs(3600)).await;
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

    let no_sound = SimpleSoundSpec {
        name: String::new(),
        gain: 1.0,
        pitch: 1.0,
        fade: 1.0,
    };
    let sound_footstep = no_sound.clone();
    let sound_dig = no_sound.clone();
    let sound_dug = no_sound.clone();

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
