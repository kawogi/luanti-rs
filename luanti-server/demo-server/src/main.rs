//! Luanti demo server implemented in Rust

use anyhow::Context;
use anyhow::bail;
use clap::ArgGroup;
use clap::Parser;
use flexstr::SharedStr;
use log::info;
use luanti_protocol::commands::client_to_server::DamageSpec;
use luanti_protocol::commands::client_to_server::InteractSpec;
use luanti_protocol::commands::client_to_server::InventoryActionSpec;
use luanti_protocol::commands::client_to_server::InventoryFieldsSpec;
use luanti_protocol::commands::client_to_server::ModchannelJoinSpec;
use luanti_protocol::commands::client_to_server::ModchannelLeaveSpec;
use luanti_protocol::commands::client_to_server::NodemetaFieldsSpec;
use luanti_protocol::commands::client_to_server::PlayerItemSpec;
use luanti_protocol::commands::client_to_server::PlayerPosCommand;
use luanti_protocol::commands::client_to_server::RespawnSpec;
use luanti_protocol::commands::client_to_server::TSChatMessageSpec;
use luanti_protocol::commands::client_to_server::TSModchannelMsgSpec;
use luanti_protocol::types::AlignStyle;
use luanti_protocol::types::AlphaMode;
use luanti_protocol::types::ContentFeatures;
use luanti_protocol::types::DrawType;
use luanti_protocol::types::InventoryAction;
use luanti_protocol::types::LiquidType;
use luanti_protocol::types::NodeDefManager;
use luanti_protocol::types::PlayerPos;
use luanti_protocol::types::PointabilityType;
use luanti_protocol::types::SColor;
use luanti_protocol::types::TileAnimationParams;
use luanti_protocol::types::TileDef;
use luanti_server::api::FromPluginEvent;
use luanti_server::api::ToPluginEvent;
use luanti_server::authentication::dummy::DummyAuthenticator;
use luanti_server::server::LuantiWorldServer;
use luanti_server::world::content_id_map::ContentIdMap;
use luanti_server::world::generation::flat::MapgenFlat;
use luanti_server::world::map_block_provider::MapBlockProvider;
use luanti_server::world::map_block_router::MapBlockRouter;
use luanti_server::world::media_registry::MediaRegistry;
use luanti_server::world::storage::minetestworld::MinetestworldStorage;
use pyo3::Python;
use pyo3::types::PyAnyMethods;
use pyo3::types::PyModule;
use std::array;
use std::ffi::CString;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::mpsc::UnboundedSender;

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

    let (to_plugin_event_sender, to_plugin_event_receiver) = mpsc::unbounded_channel();
    let (from_plugin_event_sender, from_plugin_event_receiver) = mpsc::unbounded_channel();

    API_SENDER.lock().unwrap().sender = Some(from_plugin_event_sender);

    let args = Args::parse();

    let _python_thread = thread::spawn(|| {
        if let Err(error) = run_python(to_plugin_event_receiver) {
            log::error!("Python runner returned with error: {error}");
        }
    });

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
        to_plugin_event_sender,
        from_plugin_event_receiver,
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

    // python_thread.join().unwrap();
}

static API_SENDER: Mutex<ApiSender> = Mutex::new(ApiSender::new());

struct ApiSender {
    sender: Option<UnboundedSender<FromPluginEvent>>,
}

impl ApiSender {
    const fn new() -> Self {
        Self { sender: None }
    }

    fn send(&self, event: FromPluginEvent) {
        let Some(sender) = &self.sender else {
            log::error!("API sender not initialized");
            return;
        };

        if sender.send(event).is_err() {
            log::error!("failed to send API event to engine");
        }
    }
}

fn run_python(mut receiver: UnboundedReceiver<ToPluginEvent>) -> anyhow::Result<()> {
    pyo3::append_to_inittab!(luanti);
    Python::attach(|py| {
        let sys = py.import("sys")?;
        let version: String = sys.getattr("version")?.extract()?;
        info!("running Python {version}");

        // let locals = [("os", py.import("os")?)].into_py_dict(py)?;
        // let user: String = py.eval(code, None, Some(&locals))?.extract()?;
        let code = CString::new(include_str!("../plugin.py"))?;

        let module = PyModule::from_code(py, &code, c"plugin.py", c"")?;
        let on_modchannel_join_fn = module.getattr("on_modchannel_join")?;
        let on_modchannel_leave_fn = module.getattr("on_modchannel_leave")?;
        let on_ts_modchannel_msg_fn = module.getattr("on_ts_modchannel_msg")?;
        let on_playerpos_fn = module.getattr("on_playerpos")?;
        let on_inventory_action_move_fn = module.getattr("on_inventory_action_move")?;
        let on_inventory_action_craft_fn = module.getattr("on_inventory_action_craft")?;
        let on_inventory_action_drop_fn = module.getattr("on_inventory_action_drop")?;
        let on_ts_chat_message_fn = module.getattr("on_ts_chat_message")?;
        let on_damage_fn = module.getattr("on_damage")?;
        let on_player_item_fn = module.getattr("on_player_item")?;
        let on_respawn_fn = module.getattr("on_respawn")?;
        let on_interact_fn = module.getattr("on_interact")?;
        let on_nodemeta_fields_fn = module.getattr("on_nodemeta_fields")?;
        let on_inventory_fields_fn = module.getattr("on_inventory_fields")?;

        // let t = py.run(&code, None, None)?;
        // let user: String = py.eval(&code, None, None)?.extract()?;

        while let Some(event) = receiver.blocking_recv() {
            log::trace!("received plugin event: {event:?}");

            let response = match event {
                ToPluginEvent::ModchannelJoin(ModchannelJoinSpec { channel_name }) => {
                    on_modchannel_join_fn.call1((channel_name,))
                }
                ToPluginEvent::ModchannelLeave(ModchannelLeaveSpec { channel_name }) => {
                    on_modchannel_leave_fn.call1((channel_name,))
                }
                ToPluginEvent::TSModchannelMsg(TSModchannelMsgSpec {
                    channel_name,
                    channel_msg,
                }) => on_ts_modchannel_msg_fn.call1((channel_name, channel_msg)),
                ToPluginEvent::Playerpos(PlayerPosCommand {
                    player_pos:
                        PlayerPos {
                            position,
                            speed,
                            pitch,
                            yaw,
                            keys_pressed,
                            fov,
                            wanted_range,
                            camera_inverted,
                            movement_speed,
                            movement_direction,
                        },
                }) => on_playerpos_fn.call0(),
                ToPluginEvent::InventoryAction(InventoryActionSpec { action }) => match action {
                    InventoryAction::Move {
                        count,
                        from_inv,
                        from_list,
                        from_i,
                        to_inv,
                        to_list,
                        to_i,
                    } => on_inventory_action_move_fn.call0(),
                    InventoryAction::Craft { count, craft_inv } => {
                        on_inventory_action_craft_fn.call0()
                    }
                    InventoryAction::Drop {
                        count,
                        from_inv,
                        from_list,
                        from_i,
                    } => on_inventory_action_drop_fn.call0(),
                },
                ToPluginEvent::TSChatMessage(TSChatMessageSpec { message }) => {
                    on_ts_chat_message_fn.call1((message,))
                }
                ToPluginEvent::Damage(DamageSpec { damage }) => on_damage_fn.call1((damage,)),
                ToPluginEvent::PlayerItem(PlayerItemSpec { item }) => {
                    on_player_item_fn.call1((item,))
                }
                ToPluginEvent::Respawn(RespawnSpec) => on_respawn_fn.call0(),
                ToPluginEvent::Interact(InteractSpec {
                    action,
                    item_index,
                    pointed_thing,
                    player_pos,
                }) => on_interact_fn.call0(),
                ToPluginEvent::NodemetaFields(NodemetaFieldsSpec {
                    p,
                    form_name,
                    fields,
                }) => on_nodemeta_fields_fn.call0(),
                ToPluginEvent::InventoryFields(InventoryFieldsSpec {
                    client_formspec_name,
                    fields,
                }) => on_inventory_fields_fn.call0(),
            };

            if let Err(error) = response {
                log::error!("failed to handle event: {error}");
            }
        }

        Ok(())
    })
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

#[pyo3::pymodule]
mod luanti {

    use luanti_protocol::commands::server_to_client::FovSpec;
    use luanti_server::api::FromPluginEvent;
    use pyo3::prelude::*;

    use crate::API_SENDER;

    #[pyfunction]
    fn fov(fov: f32) {
        API_SENDER
            .lock()
            .unwrap()
            .send(FromPluginEvent::Fov(FovSpec {
                fov,
                is_multiplier: true,
                transition_time: Some(1.0),
            }));
    }

    #[pyfunction]
    fn inv() {
        API_SENDER
            .lock()
            .unwrap()
            .send(FromPluginEvent::ShowFormspec(
                luanti_protocol::commands::server_to_client::ShowFormspecSpec {
                    form_spec: "size[8,7.5]
image[1,0.6;1,2;player.png]
list[current_player;main;0,3.5;8,4;]
list[current_player;craft;3,0;3,3;]
list[current_player;craftpreview;7,1;1,1;]"
                        .into(),
                    form_name: "my_formspec".into(),
                },
            ));
    }
}
