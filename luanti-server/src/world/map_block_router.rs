//! Contains `MapBlockRouter`

use std::{
    collections::{HashMap, hash_map::Entry},
    mem,
    thread::{self, JoinHandle},
    time::Duration,
};

use anyhow::Result;
use flexstr::SharedStr;
use log::{debug, error, trace, warn};
use luanti_core::MapBlockPos;
use tokio::sync::mpsc::{self, error::TryRecvError};

use super::{WorldBlock, WorldUpdate, priority::Priority, view_tracker::BlockInterest};

/// Handles map block requests from multiple players and combines them according to their priority.
/// The requests will be forwarded to a `MapBlockProvider` which will load or generate those blocks.
/// The resulting blocks will then be forwarded to the players.
pub struct MapBlockRouter {
    _runner: JoinHandle<Result<()>>,
}

impl MapBlockRouter {
    /// Creates a new [`MapBlockRouter`].
    #[must_use]
    pub fn new(
        block_request_sender: mpsc::UnboundedSender<BlockInterest>,
        world_update_receiver: mpsc::UnboundedReceiver<WorldUpdate>,
        block_interest_receiver: mpsc::UnboundedReceiver<ToRouterMessage>,
    ) -> Self {
        let runner = thread::spawn(move || {
            Self::run(
                block_interest_receiver,
                world_update_receiver,
                &block_request_sender,
            )
            .inspect_err(|error| {
                error!("router exited with error: {error}");
            })
        });

        Self {
            // block_interest_sender,
            _runner: runner,
        }
    }

    pub(crate) fn run(
        mut block_interest_receiver: mpsc::UnboundedReceiver<ToRouterMessage>,
        mut world_update_receiver: mpsc::UnboundedReceiver<WorldUpdate>,
        block_request_sender: &mpsc::UnboundedSender<BlockInterest>,
    ) -> Result<()> {
        let mut players = HashMap::new();
        let mut block_subscriptions: HashMap<MapBlockPos, EffectiveBlockInterest> = HashMap::new();
        'thread_loop: loop {
            // used to measure activity
            let mut event_count = 0;
            let mut subscription_change_count = 0;

            while let Some(message) = match block_interest_receiver.try_recv() {
                Ok(message) => {
                    event_count += 1;
                    Some(message)
                }
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => {
                    debug!("no more block interest senders exist");
                    break 'thread_loop;
                }
            } {
                match message {
                    ToRouterMessage::Register { player_key, sender } => {
                        if players.insert(player_key.clone(), sender).is_some() {
                            warn!("player '{player_key}' is already subscribed");
                        }
                    }
                    ToRouterMessage::Unregister(player_key) => {
                        if players.remove(&player_key).is_none() {
                            warn!("player '{player_key}' never subscribed");
                        }
                    }
                    ToRouterMessage::BlockInterest(BlockInterest {
                        player_key,
                        pos,
                        priority,
                    }) => {
                        if block_subscriptions
                            .entry(pos)
                            .or_default()
                            .update_player(&player_key, priority)
                        {
                            subscription_change_count += 1;
                        }
                    }
                }
            }

            while let Some(message) = match world_update_receiver.try_recv() {
                Ok(message) => {
                    event_count += 1;
                    Some(message)
                }
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => break 'thread_loop,
            } {
                // FIXME(kawogi) until the player has received this block, it might continue to send interests for that block which will eventually result in multiple map block messages
                #[expect(irrefutable_let_patterns, reason = "more variants will be added")]
                if let &WorldUpdate::NewMapBlock(WorldBlock { pos, .. }) = &message {
                    match block_subscriptions.entry(pos) {
                        Entry::Occupied(occupied_entry) => {
                            let interest = occupied_entry.remove();

                            for (player_key, _priority) in interest.player_priorities {
                                if let Some(to_player) = players.get(&player_key) {
                                    // TODO(kawogi) cloning is mad expensive. There should be a way to use an Arc internally
                                    to_player.send(message.clone())?;
                                } else {
                                    warn!("cannot forward block {pos} to player '{player_key}'");
                                }
                            }
                        }
                        Entry::Vacant(_vacant_entry) => {
                            trace!(
                                "generated block {pos} is unknown to the router and will be ignored"
                            );
                        }
                    }
                }
            }

            if subscription_change_count > 0 {
                for (pos, priority) in
                    block_subscriptions
                        .iter_mut()
                        .filter_map(|(&pos, interest)| {
                            interest.ack_max().map(|priority| (pos, priority))
                        })
                {
                    block_request_sender.send(BlockInterest {
                        player_key: SharedStr::empty(),
                        pos,
                        priority,
                    })?;
                }
            }

            // slow down event polling if there was nothing to do in the recent iteration
            if event_count == 0 {
                thread::sleep(Duration::from_millis(50));
            }
        }

        Ok(())
    }
}

#[derive(Default)]
struct EffectiveBlockInterest {
    max_priority: Priority,
    max_has_changed: bool,
    player_priorities: Vec<(SharedStr, Priority)>,
}

impl EffectiveBlockInterest {
    fn update_player(&mut self, player_key: &SharedStr, priority: Priority) -> bool {
        let max_priority = if priority.is_none() {
            // remove the player from the priority list and recompute the maximum value
            let mut new_max_priority = Priority::NONE;
            self.player_priorities
                .retain(|&(ref key, retained_priority)| {
                    let retain_entry = key != player_key;
                    if retain_entry {
                        new_max_priority = new_max_priority.max(retained_priority);
                    }
                    !retain_entry
                });
            // free up some space on the heap if the list became empty
            // (newly created empty vectors with a capacity of 0 don't require allocations)
            if self.player_priorities.is_empty() {
                drop(mem::take(&mut self.player_priorities));
            }
            new_max_priority
        } else if priority >= self.max_priority {
            // the new priority is guaranteed to overwrite the existing maximum value
            // just update/insert the entry skipping the recomputation of max value
            if let Some(player_priority) = self
                .player_priorities
                .iter_mut()
                .find_map(|(key, player_priority)| (key == player_key).then_some(player_priority))
            {
                *player_priority = priority;
            } else {
                self.player_priorities.push((player_key.clone(), priority));
            }
            priority
        } else {
            // the priority change could potentially lower the computed maximum value
            'compute_max: {
                let mut new_max_priority = Priority::NONE;
                let mut updated = false;
                for (existing_key, existing_priority) in &mut self.player_priorities {
                    if existing_key == player_key {
                        // this stunt overwrites the previous priority with the new one and creates a
                        // binding preserving the old one.
                        let existing_priority = mem::replace(existing_priority, priority);
                        updated = true;
                        // check whether the priority we just removed actually might have influenced the old max value
                        if existing_priority < self.max_priority {
                            // nope, let's abort the update and just use the the old max value
                            break 'compute_max self.max_priority;
                        }
                        assert_eq!(
                            existing_priority, self.max_priority,
                            "priority may never exceed the computed maximum"
                        );
                        // continue recomputation of the new maximum value using the new priority
                        new_max_priority = new_max_priority.max(priority);
                    } else {
                        new_max_priority = new_max_priority.max(*existing_priority);
                    }
                }

                if updated {
                    // something non-trivial changed; use the newly computed maximum
                    new_max_priority
                } else {
                    // nothing changed; add the player to this list
                    assert_eq!(
                        self.max_priority, new_max_priority,
                        "if nothing was updated the maximum value should've remained the same"
                    );
                    self.player_priorities.push((player_key.clone(), priority));
                    self.max_priority.max(priority)
                }
            }
        };

        let max_has_changed = max_priority != self.max_priority;
        if max_has_changed {
            self.max_priority = max_priority;
            self.max_has_changed = true;
        }
        max_has_changed
    }

    /// If the maximum has changed since the recent call, the new maximum will be returned
    fn ack_max(&mut self) -> Option<Priority> {
        self.max_has_changed.then(|| {
            self.max_has_changed = false;
            self.max_priority
        })
    }
}

/// Any message that can be sent from a `ViewTracker` to a `MapBlockRouter`.
pub enum ToRouterMessage {
    /// This is the first message to register a new player with the router.
    Register {
        /// Name of the player
        player_key: SharedStr,
        /// The channel to send back loaded map blocks
        sender: mpsc::UnboundedSender<WorldUpdate>,
    },
    /// This is the last message used to unregister an existing player
    Unregister(SharedStr),
    /// Tells the router about which blocks a player is interested in and how important that block
    /// is to the player.
    BlockInterest(BlockInterest),
}
