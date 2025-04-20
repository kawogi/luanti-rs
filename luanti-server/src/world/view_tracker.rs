//! Contains a task which tracks a players movement and keeps track of the blocks the shall be
//! sent to them.

use std::{
    collections::{HashMap, hash_map::Entry},
    thread::{self, JoinHandle},
    time::Duration,
};

use anyhow::Result;
use flexstr::SharedStr;
use glam::I16Vec3;
use log::{debug, error, trace, warn};
use luanti_core::MapBlockPos;
use luanti_protocol::{
    commands::client_to_server::{DeletedblocksSpec, GotBlocksSpec},
    types::PlayerPos,
};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender, error::TryRecvError};

use crate::world::WorldUpdate;

use super::{map_block_router::ToRouterMessage, priority::Priority};

/// Keeps track of the map blocks a single player is and shall be aware of.
pub(crate) struct ViewTracker {
    _player_key: SharedStr,
    _runner: JoinHandle<Result<()>>,
    player_view_sender: UnboundedSender<PlayerViewEvent>,
}

impl ViewTracker {
    pub(crate) fn new(
        player_key: SharedStr,
        block_interest_sender: UnboundedSender<ToRouterMessage>,
        world_update_sender: UnboundedSender<WorldUpdate>,
    ) -> Result<Self> {
        let (player_view_sender, player_view_receiver) = mpsc::unbounded_channel();
        let (external_world_update_sender, world_update_receiver) = mpsc::unbounded_channel();

        block_interest_sender.send(ToRouterMessage::Register {
            player_key: player_key.clone(),
            sender: external_world_update_sender,
        })?;

        // the implementation is expected to be compute intensive, so a dedicated thread should be
        // more appropriate than an async task
        let player_key_clone = player_key.clone();
        let runner = thread::spawn(move || {
            Self::run_inner(
                &player_key_clone,
                player_view_receiver,
                &block_interest_sender,
                world_update_receiver,
                &world_update_sender,
            )
            .inspect_err(|error| {
                error!("view tracker for player '{player_key_clone}' exited with error: {error}");
            })
        });

        Ok(Self {
            _player_key: player_key,
            _runner: runner,
            player_view_sender,
        })
    }

    pub(crate) fn update_view(&self, player_view_event: PlayerViewEvent) -> Result<()> {
        self.player_view_sender.send(player_view_event)?;
        Ok(())
    }

    /// - `player_view_receiver`: informs this tracker about player movements
    /// - `block_interest_sender`: reports which map blocks this player is interested in
    /// - `world_update_receiver`: informs this tracker about world updates (new blocks, changed nodes, etc.)
    /// - `world_update_sender`: used to forward changes of the world to the player
    /// - `map_block_states`: state of all map blocks the player is interested in
    #[expect(clippy::too_many_lines, reason = "//TODO(kawogi) split this up")]
    fn run_inner(
        player_key: &SharedStr,
        mut player_view_receiver: UnboundedReceiver<PlayerViewEvent>,
        block_interest_sender: &UnboundedSender<ToRouterMessage>,
        mut world_update_receiver: UnboundedReceiver<WorldUpdate>,
        world_update_sender: &UnboundedSender<WorldUpdate>,
    ) -> Result<()> {
        let mut map_block_states = HashMap::with_capacity(1024);
        let mut recent_player_block_pos = None;

        'thread_loop: loop {
            // used to measure activity
            let mut event_count = 0;

            // process bursts of user movements
            while let Some(event) = match player_view_receiver.try_recv() {
                Ok(event) => {
                    event_count += 1;
                    Some(event)
                }
                Err(TryRecvError::Disconnected) => {
                    debug!("The sender closed the view event channel for player '{player_key}'");
                    break 'thread_loop;
                }
                Err(TryRecvError::Empty) => None,
            } {
                match event {
                    PlayerViewEvent::PlayerPos(PlayerPos {
                        position,
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
                        // TODO(kawogi) this entire implementation is a placeholder and shall be replaced

                        // find the block containing the player
                        let current_block_pos = MapBlockPos::for_vec(position.round().as_i16vec3());

                        // only recompute if the player moved into a different block …
                        // this will always evaluate to `true` for the first iteration
                        if Some(current_block_pos) != recent_player_block_pos {
                            if let Some(recent_block_pos) = recent_player_block_pos {
                                trace!(
                                    "player '{player_key}' moved from block {recent_block_pos} to {current_block_pos}",
                                );
                            } else {
                                trace!("player '{player_key}' starts at block {current_block_pos}");
                            }
                            recent_player_block_pos = Some(current_block_pos);

                            // make sure that all surrounding blocks have an entry in the state table
                            let radius = 1;
                            let range = -radius..=radius;
                            for dz in range.clone() {
                                for dy in range.clone() {
                                    for dx in range.clone() {
                                        if let Some(block_pos) =
                                            current_block_pos.checked_add(I16Vec3::new(dx, dy, dz))
                                        {
                                            map_block_states
                                                .entry(block_pos)
                                                .or_insert_with(MapBlockState::default);
                                        }
                                    }
                                }
                            }

                            for (&block_pos, state) in &mut map_block_states {
                                let priority = Priority::from_block_distance(
                                    current_block_pos,
                                    block_pos,
                                    100,
                                );

                                let interest = BlockInterest::subscribe(
                                    player_key.clone(),
                                    block_pos,
                                    priority,
                                );

                                block_interest_sender
                                    .send(ToRouterMessage::BlockInterest(interest))?;
                            }
                        }
                    }
                    PlayerViewEvent::GotMapBlocks(GotBlocksSpec { blocks }) => {
                        Self::handle_got_map_blocks(player_key, &mut map_block_states, blocks);
                    }
                    PlayerViewEvent::DroppedBlocks(DeletedblocksSpec { blocks }) => {
                        Self::handle_deleted_map_blocks(
                            player_key,
                            &mut map_block_states,
                            blocks,
                            block_interest_sender,
                        )?;
                    }
                }
            }

            // process bursts of world updates
            while let Some(event) = match world_update_receiver.try_recv() {
                Ok(event) => {
                    event_count += 1;
                    Some(event)
                }
                Err(TryRecvError::Disconnected) => {
                    debug!(
                        "The sender closed the world update event channel for player '{player_key}'"
                    );
                    break 'thread_loop;
                }
                Err(TryRecvError::Empty) => None,
            } {
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
                        world_update_sender.send(WorldUpdate::NewMapBlock(world_block))?;
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
        mut blocks: Vec<I16Vec3>,
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
        mut blocks: Vec<I16Vec3>,
        block_interest_sender: &UnboundedSender<ToRouterMessage>,
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
            block_interest_sender.send(ToRouterMessage::BlockInterest(
                BlockInterest::unsubscribe(player_key.clone(), block_pos),
            ))?;
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
    pub(crate) player_key: SharedStr,
    /// position of the block
    pub(crate) pos: MapBlockPos,
    /// a value of _how much_ the player wants to see this block
    pub(crate) priority: Priority,
}

impl BlockInterest {
    fn subscribe(player_key: SharedStr, pos: MapBlockPos, priority: Priority) -> Self {
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
            priority: Priority::NONE,
        }
    }
}

#[derive(Clone, Copy, Default)]
struct MapBlockState {
    /// How important it is that the player sees this map block
    priority: Priority,
    /// whether this block has been sent to the client
    sent_to_client: bool,
    /// whether the client confirmed to have a copy of this map block
    cached_by_client: bool,
}
