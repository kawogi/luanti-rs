use anyhow::Result;
use anyhow::bail;
use log::debug;
use luanti_protocol::LuantiConnection;
use luanti_protocol::commands::CommandProperties;
use luanti_protocol::commands::client_to_server::InteractSpec;
use luanti_protocol::commands::client_to_server::PlayerPosCommand;
use luanti_protocol::commands::client_to_server::ToServerCommand;
use luanti_protocol::commands::client_to_server::UpdateClientInfoSpec;
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
            ToServerCommand::Gotblocks(_gotblocks_spec) => {
                todo!();
            }
            ToServerCommand::Deletedblocks(_deletedblocks_spec) => {
                todo!();
            }
            ToServerCommand::InventoryAction(_inventory_action_spec) => {
                todo!();
            }
            ToServerCommand::TSChatMessage(_tschat_message_spec) => {
                todo!();
            }
            ToServerCommand::Damage(_damage_spec) => {
                todo!();
            }
            ToServerCommand::Playeritem(_playeritem_spec) => {
                todo!();
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
            ToServerCommand::RequestMedia(_request_media_spec) => {
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
}
