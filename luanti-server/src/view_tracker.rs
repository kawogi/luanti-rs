//! Contains a task which tracks a players movement and keeps track of the blocks the shall be
//! sent to them.

use core::f32;
use std::{
    collections::{HashMap, hash_map::Entry},
    thread::{self, JoinHandle},
    time::Duration,
};

use anyhow::Result;
use flexstr::SharedStr;
use glam::{I16Vec3, Vec3};
use log::{debug, error, trace, warn};
use luanti_core::{MapBlockPos, MapNodePos};
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
        thread::spawn(move || {
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

        let mut recent_player_block_pos = None;

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
                        let current_block_pos = MapBlockPos::for_pos(position.round().as_i16vec3());

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
                            for dz in -2..=2 {
                                for dy in -2..=2 {
                                    for dx in -2..=2 {
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

                                block_interest_sender.blocking_send(interest)?;
                            }
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
    priority: Priority,
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

/// Represents a value describing how important something (e.g. a map block) is to the player.
///
/// This value may be used as key for a priority queue, where smaller values mean higher priority.
///
/// Priorities are coarse-grained and a conversion from float values is supported.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
struct Priority(u16);

impl Priority {
    /// This is the maximum achievable priority
    pub(crate) const MAX: Self = Self(u16::MIN);

    /// This is the maximum achievable priority (before being `NONE`)
    pub(crate) const MIN: Self = Self(Self::NONE.0 - 1);

    /// This value means that the associated object shall no longer being considered at all.
    ///
    /// The meaning is equivalent to `Option::<Priority>::None`, but without requiring an extra byte
    /// and it automatically causes `Ord` to be implemented correctly.
    pub(crate) const NONE: Self = Self(u16::MAX);

    pub(crate) fn is_none(self) -> bool {
        self == Self::NONE
    }

    pub(crate) fn is_some(self) -> bool {
        !self.is_none()
    }

    /// Uses the euclidean distance between two positions as priority.
    /// Distances exceeding `max_distance` will be mapped to `Priority::NONE`. Set the limit to
    /// `u32::MAX` to disable this.
    /// Distances exceeding the limit of `u16` will be clamped to a priority of `Priority::MIN`.
    pub(crate) fn from_vec_distance(pos_a: I16Vec3, pos_b: I16Vec3, max_distance: u32) -> Self {
        // TODO(kawogi) find a more performant solution; a very coarse approximation would be sufficient
        // note: do not use `distance_squared` because the decimation for lower distances will cause
        // all low distances to be mapped to `Priority::MAX`
        let distance = Vec3::distance(pos_a.as_vec3(), pos_b.as_vec3());
        #[expect(
            clippy::cast_precision_loss,
            reason = "the expected range is precise enough"
        )]
        if distance < max_distance as f32 {
            #[expect(clippy::cast_possible_truncation, reason = "truncation is on purpose")]
            #[expect(clippy::cast_sign_loss, reason = "distance is always positive")]
            Priority((distance as u16).max(Self::MIN.0))
        } else {
            Priority::NONE
        }
    }

    pub(crate) fn from_node_distance(
        pos_a: MapNodePos,
        pos_b: MapNodePos,
        max_distance: u32,
    ) -> Self {
        Self::from_vec_distance(pos_a.into(), pos_b.into(), max_distance)
    }

    pub(crate) fn from_block_distance(
        pos_a: MapBlockPos,
        pos_b: MapBlockPos,
        max_distance: u32,
    ) -> Self {
        Self::from_node_distance(pos_a.into(), pos_b.into(), max_distance)
    }
}

/// Converts float values in the range of `0.0..=1.0` to priorities `MAX..=MIN`.
///
/// Smaller numbers are being interpreted as higher priority, with 0.0 being the highest.
/// Negative values are being clamped to 0.0.
/// `NAN` is mapped to `Self::NONE`.
impl From<f32> for Priority {
    fn from(value: f32) -> Self {
        if value.is_nan() {
            Self::NONE
        } else {
            Self((value.clamp(0.0, 1.0) * f32::from(Self::MIN.0)) as _)
        }
    }
}

#[derive(Clone, Copy)]
struct MapBlockState {
    /// How important it is that the player sees this map block
    priority: f32,
    /// whether this block has been sent to the client
    sent_to_client: bool,
    /// whether the client confirmed to have a copy of this map block
    cached_by_client: bool,
}

impl Default for MapBlockState {
    fn default() -> Self {
        Self {
            priority: f32::INFINITY,
            sent_to_client: false,
            cached_by_client: false,
        }
    }
}
