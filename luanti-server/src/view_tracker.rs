//! Contains a task which tracks a players movement and keeps track of the blocks the shall be
//! sent to them.

use std::thread::JoinHandle;

use anyhow::Result;
use flexstr::SharedStr;
use glam::Vec3;
use log::{debug, error, trace};
use luanti_core::MapBlockPos;
use luanti_protocol::{
    commands::client_to_server::{DeletedblocksSpec, GotBlocksSpec},
    types::{PlayerPos, TransferrableMapBlock},
};
use tokio::sync::mpsc::{Receiver, Sender};

struct ViewTracker {
    player_key: SharedStr,
    player_view_events: Receiver<PlayerViewEvent>,
    block_interest_events: Sender<BlockInterest>,
    world_update_events: Receiver<WorldUpdate>,
}

impl ViewTracker {
    pub(crate) fn new(
        player_key: SharedStr,
        player_view_events: Receiver<PlayerViewEvent>,
        block_interest_events: Sender<BlockInterest>,
        world_update_events: Receiver<WorldUpdate>,
    ) -> Self {
        Self {
            player_key,
            player_view_events,
            block_interest_events,
            world_update_events,
        }
    }

    pub(crate) fn run(self) -> JoinHandle<Result<()>> {
        let player_key = self.player_key.clone();
        // the implementation is expected to be compute intensive, so a dedicated thread should be
        // more appropriate than an async task
        std::thread::spawn(move || {
            self.run_inner().inspect_err(|error| {
                error!("view tracker for player '{player_key}' exited with error: {error}");
            })
        })
    }

    fn run_inner(self) -> Result<()> {
        let Self {
            player_key,
            mut player_view_events,
            block_interest_events,
            world_update_events,
        } = self;

        let mut player_position = Vec3::new(f32::NAN, f32::NAN, f32::NAN);

        'next_view_event: loop {
            let Some(event) = player_view_events.blocking_recv() else {
                debug!("The sender closed the event channel for player '{player_key}'");
                break 'next_view_event;
            };
            match event {
                PlayerViewEvent::PlayerPos(PlayerPos {
                    position: new_position,
                    speed: _,
                    pitch: _,
                    yaw: _,
                    keys_pressed: _,
                    fov: _,
                    wanted_range: _,
                    camera_inverted: _,
                    movement_speed: _,
                    movement_direction: _,
                }) => {
                    // only recompute if the player moved a noteworthy distance …
                    if new_position.distance_squared(player_position) > 4.0 * 4.0 {
                        trace!("player moved from {player_position} to {new_position}");
                        player_position = new_position;
                    }
                }
                PlayerViewEvent::GotMapBlocks(GotBlocksSpec { mut blocks }) => {
                    for block_pos in blocks.drain(..).filter_map(MapBlockPos::new) {
                        block_interest_events.blocking_send(BlockInterest::unsubscribe(
                            player_key.clone(),
                            block_pos,
                        ));
                    }
                }
                PlayerViewEvent::DroppedBlocks(DeletedblocksSpec { mut blocks }) => {
                    for block_pos in blocks.drain(..).filter_map(MapBlockPos::new) {
                        block_interest_events.blocking_send(BlockInterest::unsubscribe(
                            player_key.clone(),
                            block_pos,
                        ));
                    }
                }
            }
        }

        Ok(())
    }
}

pub(crate) enum PlayerViewEvent {
    /// The player has changed its position or viewing direction
    PlayerPos(PlayerPos),
    /// The player confirmed to have received some blocks
    GotMapBlocks(GotBlocksSpec),
    /// The player reports to have removed some map blocks from its cache
    DroppedBlocks(DeletedblocksSpec),
}

pub(crate) struct BlockInterest {
    player_key: SharedStr,
    /// position of the block
    pos: MapBlockPos,
    /// a value of _how much_ the player wants to see this block
    priority: f32,
}

impl BlockInterest {
    fn new(player_key: SharedStr, pos: MapBlockPos, priority: f32) -> Self {
        Self {
            player_key,
            pos,
            priority,
        }
    }

    fn unsubscribe(player_key: SharedStr, pos: MapBlockPos) -> Self {
        Self {
            player_key,
            pos,
            priority: f32::NAN,
        }
    }
}

struct BlockState {
    priority: f32,
    /// the version that has been sent to the client
    sent_to_client: u64,
    /// whether the client confirmed to have a copy of this map block
    cached_by_client: bool,
}

pub(crate) enum WorldUpdate {
    NewMapBlock(TransferrableMapBlock),
}
