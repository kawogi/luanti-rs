use std::{
    collections::HashMap,
    mem,
    thread::{self, JoinHandle},
};

use anyhow::{Result, bail};
use flexstr::SharedStr;
use log::{error, warn};
use luanti_core::MapBlockPos;
use tokio::sync::mpsc;

use super::{WorldUpdate, priority::Priority, view_tracker::BlockInterest};

pub(crate) struct MapBlockRouter {
    block_interest_sender: mpsc::UnboundedSender<ToRouterMessage>,
    block_request_sender: mpsc::UnboundedSender<BlockInterest>,
    runner: JoinHandle<Result<()>>,
}

impl MapBlockRouter {
    pub(crate) fn new(block_request_sender: mpsc::UnboundedSender<BlockInterest>) -> Self {
        let (block_interest_sender, block_interest_receiver) = mpsc::unbounded_channel();

        let runner = thread::spawn(|| {
            Self::run(block_interest_receiver).inspect_err(|error| {
                error!("router exited with error: {error}");
            })
        });

        Self {
            block_interest_sender,
            block_request_sender,
            runner,
        }
    }

    pub(crate) fn run(
        mut block_interest_receiver: mpsc::UnboundedReceiver<ToRouterMessage>,
    ) -> Result<()> {
        let mut players = HashMap::new();
        let mut block_subscriptions: HashMap<MapBlockPos, EffectiveBlockInterest> = HashMap::new();
        loop {
            let Some(message) = block_interest_receiver.blocking_recv() else {
                bail!("no more block interest senders exist");
            };

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
                    block_subscriptions
                        .entry(pos)
                        .or_default()
                        .update_player(&player_key, priority);
                }
            }
        }

        Ok(())
    }
}

#[derive(Default)]
struct EffectiveBlockInterest {
    max_priority: Priority,
    player_priorities: Vec<(SharedStr, Priority)>,
}

impl EffectiveBlockInterest {
    fn update_player(&mut self, player_key: &SharedStr, priority: Priority) {
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

        self.max_priority = max_priority;
    }
}

pub(crate) enum ToRouterMessage {
    Register {
        player_key: SharedStr,
        sender: mpsc::UnboundedSender<WorldUpdate>,
    },
    Unregister(SharedStr),
    BlockInterest(BlockInterest),
}
