//! Luanti demo server implemented in Rust

use anyhow::Context;
use anyhow::bail;
use clap::ArgGroup;
use clap::Parser;
use flexstr::SharedStr;
use log::info;
use luanti_protocol::types::AlignStyle;
use luanti_protocol::types::AlphaMode;
use luanti_protocol::types::ContentFeatures;
use luanti_protocol::types::DrawType;
use luanti_protocol::types::LiquidType;
use luanti_protocol::types::NodeDefManager;
use luanti_protocol::types::PointabilityType;
use luanti_protocol::types::SColor;
use luanti_protocol::types::TileAnimationParams;
use luanti_protocol::types::TileDef;
use luanti_server::authentication::dummy::DummyAuthenticator;
use luanti_server::server::LuantiWorldServer;
use luanti_server::world::content_id_map::ContentIdMap;
use luanti_server::world::generation::flat::MapgenFlat;
use luanti_server::world::map_block_provider::MapBlockProvider;
use luanti_server::world::map_block_router::MapBlockRouter;
use luanti_server::world::media_registry::MediaRegistry;
use luanti_server::world::storage::minetestworld::MinetestworldStorage;
use std::array;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

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

#[expect(clippy::too_many_lines, reason = "// TODO(kawogi) split this up")]
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
    info!("Starting demo server on {bind_addr}");

    let mut media_registry = MediaRegistry::default();
    media_registry
        .load_directory("luanti-server/demo-server/assets")
        .context("failed to load assets")?;

    let mut content_id_map = ContentIdMap::new();
    let content_id_stone = content_id_map.push(SharedStr::from_borrowed("basenodes:stone"))?;
    let content_id_sand = content_id_map.push(SharedStr::from_borrowed("basenodes:sand"))?;
    let content_id_dirt_with_grass =
        content_id_map.push(SharedStr::from_borrowed("basenodes:dirt_with_grass"))?;
    let content_id_dirt = content_id_map.push(SharedStr::from_borrowed("basenodes:dirt"))?;
    let content_id_water_source =
        content_id_map.push(SharedStr::from_borrowed("basenodes:water_source"))?;
    let content_id_water_flowing =
        content_id_map.push(SharedStr::from_borrowed("basenodes:water_flowing"))?;
    let content_id_block_of_rust =
        content_id_map.push(SharedStr::from_borrowed("demo:block_of_rust"))?;

    let tile_dirt = tile_def("demo_dirt.png");
    let tile_grass_east = tile_def("demo_grass_east.png");
    let tile_grass_north = tile_def("demo_grass_north.png");
    let tile_grass_south = tile_def("demo_grass_south.png");
    let tile_grass_west = tile_def("demo_grass_west.png");
    let tile_grass = tile_def("demo_grass.png");
    let tile_sand = tile_def("demo_sand.png");
    let tile_stone = tile_def("demo_stone.png");
    let tile_water = tile_def("demo_water.png^[opacity:160");
    let tile_rust = tile_def("rust_tile_32.png");

    let tile_none = tile_def("");
    let content_dirt = content_features("basenodes:dirt", &[&tile_dirt], &[], DrawType::Normal);
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
        DrawType::Normal,
    );
    let content_sand = content_features("basenodes:sand", &[&tile_sand], &[], DrawType::Normal);
    let content_stone = content_features("basenodes:stone", &[&tile_stone], &[], DrawType::Normal);
    let content_water_source = content_features(
        "basenodes:water_source",
        &[&tile_water],
        &[],
        DrawType::Liquid,
    );
    let content_water_flowing = content_features(
        "basenodes:water_flowing",
        &[&tile_water],
        &[],
        DrawType::Liquid,
    );
    let content_block_of_rust =
        content_features("demo:block_of_rust", &[&tile_rust], &[], DrawType::Normal);

    let node_def_manager = NodeDefManager {
        content_features: vec![
            (content_id_stone.0, content_stone),
            (content_id_sand.0, content_sand),
            (content_id_dirt_with_grass.0, content_dirt_with_grass),
            (content_id_dirt.0, content_dirt),
            (content_id_water_source.0, content_water_source),
            (content_id_water_flowing.0, content_water_flowing),
            (content_id_block_of_rust.0, content_block_of_rust),
        ],
    };

    let world_generator = MapgenFlat::new(content_id_block_of_rust);
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

    let mut server = LuantiWorldServer::new(
        bind_addr,
        args.verbose,
        Arc::new(node_def_manager),
        Arc::new(media_registry),
    );

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

fn content_features(
    name: &str,
    tiles: &[&TileDef],
    overlays: &[&TileDef],
    drawtype: DrawType,
) -> ContentFeatures {
    let tiledef = array::from_fn(|index| {
        (*tiles
            .get(index)
            .or(tiles.last())
            .unwrap_or(&&TileDef::new(String::new())))
        .clone()
    });
    let tiledef_overlay = array::from_fn(|index| {
        (*overlays
            .get(index)
            .or(overlays.last())
            .unwrap_or(&&TileDef::new(String::new())))
        .clone()
    });

    let is_water = matches!(drawtype, DrawType::Liquid);
    if is_water {
        ContentFeatures {
            drawtype,
            tiledef,
            tiledef_overlay,
            alpha_for_legacy: 160,
            waving: 3,
            post_effect_color: SColor::new(64, 100, 100, 200),
            walkable: false,
            pointable: PointabilityType::PointableNot,
            diggable: false,
            liquid_type: LiquidType::Source,
            drowning: 1,
            alpha: AlphaMode::Blend,
            liquid_move_physics: true,
            ..ContentFeatures::new_unknown(name.into())
        }
    } else {
        ContentFeatures {
            drawtype,
            tiledef,
            tiledef_overlay,
            ..ContentFeatures::new_unknown(name.into())
        }
    }
}
