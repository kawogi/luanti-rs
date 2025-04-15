//! Contains a task which tracks a players movement and keeps track of the blocks the shall be
//! sent to them.

use std::{
    collections::{HashMap, hash_map::Entry},
    thread::{self, JoinHandle},
    time::Duration,
};

use anyhow::Result;
use flexstr::SharedStr;
use glam::Vec3;
use log::{debug, error, trace, warn};
use luanti_core::MapBlockPos;
use luanti_protocol::{
    commands::client_to_server::{DeletedblocksSpec, GotBlocksSpec},
    types::PlayerPos,
};
use tokio::sync::mpsc::{Receiver, Sender, error::TryRecvError};

use crate::world::WorldUpdate;

/// Keeps track of the map blocks a single player is and shall be aware of.
struct ViewTracker {
    player_key: SharedStr,
    /// informs this tracker about player movements
    player_view_receiver: Receiver<PlayerViewEvent>,
    /// reports which map blocks this player is interested in
    block_interest_sender: Sender<BlockInterest>,
    /// informs this tracker about world updates (new blocks, changed nodes, etc.)
    world_update_receiver: Receiver<WorldUpdate>,
    /// used to forward changes of the world to the player
    world_update_sender: Sender<WorldUpdate>,
    /// state of all map blocks the player is interested in
    map_block_states: HashMap<MapBlockPos, MapBlockState>,
}

impl ViewTracker {
    pub(crate) fn new(
        player_key: SharedStr,
        player_view_receiver: Receiver<PlayerViewEvent>,
        block_interest_sender: Sender<BlockInterest>,
        world_update_receiver: Receiver<WorldUpdate>,
        world_update_sender: Sender<WorldUpdate>,
    ) -> Self {
        Self {
            player_key,
            player_view_receiver,
            block_interest_sender,
            world_update_receiver,
            world_update_sender,
            map_block_states: HashMap::with_capacity(1024),
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

    #[allow(dead_code, clippy::too_many_lines)]
    fn run_inner(self) -> Result<()> {
        let Self {
            player_key,
            mut player_view_receiver,
            block_interest_sender,
            mut world_update_receiver,
            world_update_sender,
            mut map_block_states,
        } = self;

        let mut player_position = Vec3::new(f32::NAN, f32::NAN, f32::NAN);

        'thread_loop: loop {
            // used to measure activity
            let mut event_count = 0;

            // process bursts of user movements
            while let Some(event) = match player_view_receiver.try_recv() {
                Ok(event) => Some(event),
                Err(TryRecvError::Disconnected) => {
                    debug!("The sender closed the view event channel for player '{player_key}'");
                    break 'thread_loop;
                }
                Err(TryRecvError::Empty) => None,
            } {
                event_count += 1;
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

                            // TODO compute priorities and subscribe
                        }
                    }
                    PlayerViewEvent::GotMapBlocks(GotBlocksSpec { blocks }) => {
                        Self::handle_got_map_blocks(&player_key, &mut map_block_states, blocks);
                    }
                    PlayerViewEvent::DroppedBlocks(DeletedblocksSpec { blocks }) => {
                        Self::handle_deleted_map_blocks(
                            &player_key,
                            &mut map_block_states,
                            blocks,
                            &block_interest_sender,
                        )?;
                    }
                }
            }

            // process bursts of world updates
            while let Some(event) = match world_update_receiver.try_recv() {
                Ok(event) => Some(event),
                Err(TryRecvError::Disconnected) => {
                    debug!(
                        "The sender closed the world update event channel for player '{player_key}'"
                    );
                    break 'thread_loop;
                }
                Err(TryRecvError::Empty) => None,
            } {
                event_count += 1;
                match event {
                    WorldUpdate::NewMapBlock(world_block) => {
                        let block_pos = world_block.pos;

                        match map_block_states.entry(block_pos) {
                            Entry::Occupied(mut occupied_entry) => {
                                // let mut state = occupied_entry.get_mut();
                                // if state.sent_to_client {
                                //     warn!(
                                //         "player '{player_key}' already received a copy of map block {block_pos}"
                                //     );
                                // }
                            }
                            Entry::Vacant(_vacant_entry) => {
                                // trace!(
                                //     "player '{player_key}' has no interest in map block {block_pos}"
                                // );
                            }
                        }

                        // just forward this block to the player
                        trace!(
                            "forwarding map block {pos} to player '{player_key}'",
                            pos = world_block.pos
                        );
                        world_update_sender.blocking_send(WorldUpdate::NewMapBlock(world_block))?;
                    }
                }
            }

            // slow down event polling if there was nothing to do in the recent iteration
            if event_count == 0 {
                thread::sleep(Duration::from_millis(50));
            }
        }

        Ok(())
    }

    fn handle_got_map_blocks(
        player_key: &SharedStr,
        map_block_states: &mut HashMap<MapBlockPos, MapBlockState>,
        mut blocks: Vec<glam::I16Vec3>,
    ) {
        for block_pos in blocks.drain(..).filter_map(MapBlockPos::new) {
            match map_block_states.entry(block_pos) {
                Entry::Occupied(mut occupied_entry) => match occupied_entry.get_mut() {
                    MapBlockState {
                        sent_to_client: false,
                        ..
                    } => {
                        warn!(
                            "player '{player_key}' confirmed reception of map block {block_pos}, which was never sent to them"
                        );
                    }
                    MapBlockState {
                        cached_by_client: true,
                        ..
                    } => {
                        warn!(
                            "player '{player_key}' confirmed reception of map block {block_pos}, which has already been confirmed earlier"
                        );
                    }
                    state => {
                        trace!(
                            "player '{player_key}' confirmed reception of map block {block_pos}"
                        );
                        state.cached_by_client = true;
                    }
                },
                Entry::Vacant(_vacant_entry) => {
                    warn!(
                        "player '{player_key}' confirmed reception of map block {block_pos}, which is unknown to the view tracker"
                    );
                }
            }
        }
    }

    fn handle_deleted_map_blocks(
        player_key: &SharedStr,
        map_block_states: &mut HashMap<MapBlockPos, MapBlockState>,
        mut blocks: Vec<glam::I16Vec3>,
        block_interest_sender: &Sender<BlockInterest>,
    ) -> Result<(), anyhow::Error> {
        for block_pos in blocks.drain(..).filter_map(MapBlockPos::new) {
            match map_block_states.entry(block_pos) {
                // remove state for this block
                Entry::Occupied(occupied_entry) => match occupied_entry.remove() {
                    MapBlockState {
                        sent_to_client: false,
                        ..
                    } => {
                        warn!(
                            "player '{player_key}' reported dropping of map block {block_pos}, which was never sent to them"
                        );
                    }
                    MapBlockState {
                        cached_by_client: false,
                        ..
                    } => {
                        warn!(
                            "player '{player_key}' reported dropping of map block {block_pos}, which was never confirmed to be received"
                        );
                    }
                    _ => trace!("player '{player_key}' removed map block {block_pos}"),
                },
                Entry::Vacant(_vacant_entry) => {
                    warn!(
                        "player '{player_key}' reported dropping of map block {block_pos}, which is unknown to the view tracker"
                    );
                }
            }

            // report, that we're no longer interested in updates for this map block
            block_interest_sender
                .blocking_send(BlockInterest::unsubscribe(player_key.clone(), block_pos))?;
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

#[derive(Clone, Copy)]
struct MapBlockState {
    /// How important it if that the player sees this map block
    priority: f32,
    /// whether this block has been sent to the client
    sent_to_client: bool,
    /// whether the client confirmed to have a copy of this map block
    cached_by_client: bool,
}
