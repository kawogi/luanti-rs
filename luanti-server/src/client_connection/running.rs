use anyhow::Result;
use anyhow::bail;
use log::debug;
use luanti_protocol::LuantiConnection;
use luanti_protocol::commands::CommandProperties;
use luanti_protocol::commands::client_to_server::GotBlocksSpec;
use luanti_protocol::commands::client_to_server::InteractSpec;
use luanti_protocol::commands::client_to_server::InventoryActionSpec;
use luanti_protocol::commands::client_to_server::PlayerItemSpec;
use luanti_protocol::commands::client_to_server::PlayerPosCommand;
use luanti_protocol::commands::client_to_server::TSChatMessageSpec;
use luanti_protocol::commands::client_to_server::ToServerCommand;
use luanti_protocol::commands::client_to_server::UpdateClientInfoSpec;
use luanti_protocol::types::InventoryAction;
use luanti_protocol::types::InventoryLocation;
use luanti_protocol::types::PlayerPos;
use luanti_protocol::types::PointedThing;

/// Everything has been set up. We're in-game now!
pub(super) struct RunningState;

impl RunningState {
    #[must_use]
    pub(super) fn new() -> Self {
        Self {}
    }

    pub(crate) fn handle_message(
        &mut self,
        message: ToServerCommand,
        _connection: &LuantiConnection,
    ) -> Result<()> {
        match message {
            ToServerCommand::Playerpos(player_pos_command) => {
                Self::handle_player_pos(*player_pos_command)?;
            }
            ToServerCommand::UpdateClientInfo(update_client_info_spec) => {
                Self::handle_update_client_info(&update_client_info_spec)?;
            }
            ToServerCommand::ModchannelJoin(_modchannel_join_spec) => {
                todo!();
            }
            ToServerCommand::ModchannelLeave(_modchannel_leave_spec) => {
                todo!();
            }
            ToServerCommand::TSModchannelMsg(_tsmodchannel_msg_spec) => {
                todo!();
            }
            ToServerCommand::GotBlocks(got_blocks_spec) => {
                Self::handle_got_blocks(*got_blocks_spec)?;
            }
            ToServerCommand::Deletedblocks(_deletedblocks_spec) => {
                todo!();
            }
            ToServerCommand::InventoryAction(inventory_action_spec) => {
                Self::handle_inventory_action(*inventory_action_spec)?;
            }
            ToServerCommand::TSChatMessage(tschat_message_spec) => {
                Self::handle_chat_message(*tschat_message_spec)?;
            }
            ToServerCommand::Damage(_damage_spec) => {
                todo!();
            }
            ToServerCommand::PlayerItem(player_item_spec) => {
                Self::handle_player_item(&player_item_spec)?;
            }
            ToServerCommand::Respawn(_respawn_spec) => {
                todo!();
            }
            ToServerCommand::Interact(interact_spec) => {
                Self::handle_interact(*interact_spec)?;
            }
            ToServerCommand::RemovedSounds(_removed_sounds_spec) => {
                todo!();
            }
            ToServerCommand::NodemetaFields(_nodemeta_fields_spec) => {
                todo!();
            }
            ToServerCommand::InventoryFields(_inventory_fields_spec) => {
                todo!();
            }
            ToServerCommand::HaveMedia(_have_media_spec) => {
                todo!();
            }
            ToServerCommand::FirstSrp(_first_srp_spec) => {
                todo!();
            }
            unexpected => {
                bail!(
                    "unexpected command for authenticated state: {}",
                    unexpected.command_name()
                );
            }
        }

        Ok(())
    }

    #[expect(
        clippy::unnecessary_wraps,
        clippy::needless_pass_by_value,
        reason = "//TODO(kawogi) for symmetry with other handlers, but should be reviewed"
    )]
    fn handle_player_pos(
        player_pos_command: PlayerPosCommand,
    ) -> std::result::Result<(), anyhow::Error> {
        let PlayerPosCommand {
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
        } = player_pos_command;

        debug!(
            "player moved: pos:({px},{py},{pz}) speed:({sx},{sy},{sz}) pitch:{pitch} yaw:{yaw} keys:{keys_pressed} fov:{fov} range:{wanted_range} cam_inv:{camera_inverted} mov_speed:{movement_speed} mov_dir:{movement_direction} ",
            px = position.x,
            py = position.y,
            pz = position.z,
            sx = speed.x,
            sy = speed.y,
            sz = speed.z,
        );

        Ok(())
    }

    #[expect(
        clippy::unnecessary_wraps,
        reason = "//TODO(kawogi) for symmetry with other handlers, but should be reviewed"
    )]
    fn handle_update_client_info(update_client_info_spec: &UpdateClientInfoSpec) -> Result<()> {
        let &UpdateClientInfoSpec {
            render_target_size,
            real_gui_scaling,
            real_hud_scaling,
            max_fs_size,
            touch_controls,
        } = update_client_info_spec;

        debug!(
            "updated client info: render size:({render_x},{render_y}) gui scaling:({real_gui_scaling}) hud scaling:{real_hud_scaling} fs size:({fs_x},{fs_y}) touch:{touch_controls}",
            render_x = render_target_size.x,
            render_y = render_target_size.y,
            fs_x = max_fs_size.x,
            fs_y = max_fs_size.y,
        );

        Ok(())
    }

    #[expect(
        clippy::unnecessary_wraps,
        reason = "//TODO(kawogi) for symmetry with other handlers, but should be reviewed"
    )]
    fn handle_interact(interact_spec: InteractSpec) -> Result<()> {
        let InteractSpec {
            action,
            item_index,
            pointed_thing,
            player_pos: _,
        } = interact_spec;

        let pointed_thing = match pointed_thing {
            PointedThing::Nothing => "nothing".into(),
            PointedThing::Node {
                under_surface,
                above_surface,
            } => format!("node {under_surface}:{above_surface}"),
            PointedThing::Object { object_id } => format!("object #{object_id}"),
        };

        debug!("interaction: {action:?} item:#{item_index} pointed:{pointed_thing}",);
        Ok(())
    }

    #[expect(
        clippy::unnecessary_wraps,
        reason = "//TODO(kawogi) for symmetry with other handlers, but should be reviewed"
    )]
    fn handle_player_item(player_item_spec: &PlayerItemSpec) -> Result<()> {
        let PlayerItemSpec { item } = player_item_spec;

        debug!("player item #{item}");
        Ok(())
    }

    #[expect(
        clippy::unnecessary_wraps,
        reason = "//TODO(kawogi) for symmetry with other handlers, but should be reviewed"
    )]
    fn handle_chat_message(chat_message_spec: TSChatMessageSpec) -> Result<()> {
        let TSChatMessageSpec { message } = chat_message_spec;

        debug!("chat message: '{message}'");
        Ok(())
    }

    #[expect(
        clippy::unnecessary_wraps,
        reason = "//TODO(kawogi) for symmetry with other handlers, but should be reviewed"
    )]
    fn handle_inventory_action(inventory_action_spec: InventoryActionSpec) -> Result<()> {
        let InventoryActionSpec { action } = inventory_action_spec;

        let inventory_location = |location| -> String {
            match location {
                InventoryLocation::Undefined => "undefined".into(),
                InventoryLocation::CurrentPlayer => "current_player".into(),
                InventoryLocation::Player { name } => {
                    format!("player:'{name}'")
                }
                InventoryLocation::NodeMeta { pos } => {
                    format!("node_meta@{pos}")
                }
                InventoryLocation::Detached { name } => {
                    format!("detached:'{name}'")
                }
            }
        };

        match action {
            InventoryAction::Move {
                count,
                from_inv,
                from_list,
                from_i,
                to_inv,
                to_list,
                to_i,
            } => {
                debug!(
                    "inventory move: {count}× from {from_inv}/{from_list}[{from_i}] → {to_inv}/{to_list}[{to_i}]",
                    from_inv = inventory_location(from_inv),
                    to_inv = inventory_location(to_inv),
                    to_i = to_i.as_ref().map_or("?".into(), ToString::to_string),
                );
            }
            InventoryAction::Craft { count, craft_inv } => {
                debug!(
                    "inventory craft: {count}× in {}",
                    inventory_location(craft_inv)
                );
            }
            InventoryAction::Drop {
                count,
                from_inv,
                from_list,
                from_i,
            } => {
                debug!(
                    "inventory drop: {count}× from {inv}/{from_list}[{from_i}]",
                    inv = inventory_location(from_inv)
                );
            }
        }

        Ok(())
    }

    #[expect(
        clippy::unnecessary_wraps,
        reason = "//TODO(kawogi) for symmetry with other handlers, but should be reviewed"
    )]
    fn handle_got_blocks(got_blocks_spec: GotBlocksSpec) -> Result<()> {
        let GotBlocksSpec { blocks } = got_blocks_spec;

        debug!("got blocks: {blocks:?}");

        Ok(())
    }
}
