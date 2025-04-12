# Transmission of world content to the player

This document aims to lay out a mechanism for deciding which world elements (map blocks) shall be

- generated
- loaded
- stored
- cached
- transmitted

The constraints are (partially contradicting):

- minimize generation of new blocks to save server CPU
- minimize storage
- minimize transmission to client to save bandwidth
- minimize latency

## Player interest

Each player sees a different part of the world which are ever-changing and might overlap. The exact
set of map blocks a player may see depends on:

- location in the world
- viewing direction
- viewing distance
- field of view (incl. aspect ratio)
- occlusion (map block might be invisible because they're underground)

Everything but occlusion will be sent by each player whenever something changes.
Occlusion is hard to compute but very effective in reducing the amount of blocks to be generated
and transmitted.

On top of the current input values, there's also interest in map blocks that are very likely to be
of interest in the near future:

- changing the viewing direction will bring a lot of new map blocks into view
- moving forward will eventually require to generate more map blocks towards the horizon
- moving in general (esp. sideways or jumping) require to re-compute occlusion

Other input values rarely change.

It is acceptable to see gaps in the world, but the implementation shall attempt to minimize the
disruption caused by these gaps.

### Player attention predictor (placeholder name)

For each active player there shall be a _worker_ whose job is to keep track of the player's movement
and decide which blocks might be the most important. There could be two priorities:

- how important is the map block to be seen right now (needs to be generated/loaded and transmitted)
- how important a map block might become soon (needs to be generated but not necessarily loaded or
  transmitted).

A first implementation will very likely start with a single priority value for each map block.

Implementation detail: the priority will very likely be a single `f32` (float) value where lower
values mean higher priority; `0.0` being the highest priority.

There are a few techniques that might be implemented:

- the greater the distance between a block and the player, the lower the priority
- the further off-axis a block is from the viewing direction, the lower the priority
- use ray-casting to determine which blocks are invisible; add probabilistic rays for positions and
  viewing directions that are likely (jumping, looking back)

For the raycasting to work, the predictor passively listens to the stream of generated map blocks.

### Merging Player interests

A single map block may be seen by multiple players. Its internal representation allows it to be
generated once and then send an exact copy to each player. Especially for crowded areas with lots of
players, this saves resources for generation, loading and compression.

For this to work well, the server has to combine the priorities of all players for each map block.
There are some approaches one could try:

1. increase the priority with each player
2. compute the arithmetic average
3. compute the maximum priority (minimum numeric value)
4. compute the median

Option 1 very likely exhibits the following downside: If most players gather in a single location,
they will dominate the priority computation. Players that are further off won't get enough map
blocks sent to them or at least not in time. Also the math behind that is tricky, because the
value shall shrink with each player

Option 2 has the problem of producing a too low priority for everyone if a single player adds a
value which is too big (possibly approaching `+Inf`).

Option 3 seems to be a good approach.

Option 4 is hard to predict. It's very complex to implement and it might suffer from the same
problems as Option 1.

It is important to note that even though the players individual priorities are being merged for
later processing, as soon as the generated/loaded blocks are being ready to be sent, the individual
priority shall be used to influence the order in which the map blocks shall be sent to each player.

The result of this stage is a stream of priorities that will be attached to map block locations.
The next stage will asynchronously use these priorities to decide which map blocks to generate or
load.

## Map Block Provision

A map block provider's job it to listen to a stream of map block priorities and to make sure that
the requested map blocks become available eventually.

There are three sources for map blocks:

- local cache
- loading from storage
- generation (mapgen)

### Cache lookup

Whenever a map block is requested the provider shall first check the local cache. The cache is
initially empty and will be filled by the later stages.

An optional enhancement would be to store or offload the cache to a persistent memory.
This is separate from the usual _official_ storage and merely serves as a measure to reduce the
number of CPU-intensive generations and to speed up the startup. It shall be possible to delete
this persisted cache any time.

### Storage lookup

Next the provider shall look for a stored block that can be loaded. Any loaded block will be added
to the local cache.

Depending on the storage provider it might happen that blocks are being loaded in groups even though
only a single one has been requested. Those additional blocks shall be added to the cache as well.
All storage providers should be implemented in a way that grouped blocks are spatially close to each
other, so that there's a high likelihood of those blocks being or becoming of interest as well.

### Generation

If a block is missing from the storage, a generation shall be triggered. The generation of a single
block may implicitly trigger the generation of multiple blocks (e.g. entire map chunks). All map
blocks that are being generated shall be made available to the cache as well.

The generation of all map blocks shall be deterministic and idempotent, meaning: irrespective of how
often and when a certain chunk is being generated, it shall always yield the same result. This must
also hold for mods that are being used for map generation. If a mod decides to violate this rule
(e.g. taking the previous gameplay into account), the resulting map block shall be
[_modified_](#modification) **after** the generation - this should be a different callback from the
API.

These rules permit to get rid of map blocks from the cache any time, because they can simply be
reconstructed on demand.

The generation might benefit from it's own cache. Deleting this cache shall never have a visible
side effect other than performance implications.

Note: There could be extra storage for very expensive calculations that occur on the first startup
(e.g. pre-computing biomes, rivers and other large structures).
This should be a one-time operation and never shall be updated during normal gameplay.

### Modification

Each map block shall have a version number which is monotonically increasing with each modification.

A newly generated block shall have a version of `0`. This also serves as an indicator that this
block can safely be removed from any cache as its state can be re-generated any time.

Note: maybe the first version shall be `1`, freeing up `0` for the internal representation of a
_not yet generated_ map block.

Whenever a map block is being modified the version number increases. There are two approaches:

1. the block's version number is increased by `1` for each modification
2. the world has a global version number which is increased by `1` and then assigned to the block.
3. the engine has a global tick counter which is being assigned to each block on modification

Variant 1 is simple to implement and ensures that it can be verified that no update has been
missed. If this variant isn't being used, another mechanism needs to ensure that _newly generated_
blocks become distinguishable from modified blocks.

Variant 2 enables to create a _snapshot_ and rollbacks of the entire world using a single number.

Variant 3 is similar to variant 2 but makes it easier to synchronize map block changes with other
storage systems (e.g. player storage) as the global tick counter is available to all components of
the engine, while the version from variant 2 needs to be transferred from the world generation to
other components in order to be useful. A downside compared to variant 2 would be that _snapshot_
versions are not consecutive and that multiple modifications within a single tick could become more
complicated.

As a first approach both versions from variant 1 (block local) and 3 (global tick) shall be stored
for each map block. Having variant 1 also partially solves the "multiple changes per tick" problem.
Only _partially_ because multiple changes in different block on the same tick can still not be
ordered.

Whenever a block has been modified it shall replace the old version in the cache. As a side effect
this block needs to be transferred to all clients which had access to the old version. The usual
priority rules need to be applied for scheduling and the raycasting algorithm needs to take these
changes into account.

### Persistence

Each modification to the game's state will be tracked with a version (tick counter). After the
completion of each tick the state is valid as a whole and could be persisted on a medium (files,
database, …).

There can be multiple storages for different content types (map data, chat, player, …) and all of
them need to be synchronized. For this to work the following approach shall be taken:

- decide that a certain tick shall be persisted (this can be based on a fixed time interval, the
  amount of data collected in caches, the system being idle, … and shall generally be configurable)
- after the end of each tick, notify all storage providers to take a snapshot of the current state.
  This shall be performed in a non-blocking manner and depending on the amount of data this can
  mean to:
  - just write out the data right away if there's little risk of blocking the game
  - create an in-memory copy of the current state and write that asynchronously
  - use a generational data model (clone on write) - this is being used for map blocks
- continue the game loop while the storage providers are busy
- each storage provider reports back to the engine as soon as they ensured the requested state has
  been successfully persisted.
- as soon as all storage providers reported success, the engine informs all of them about the
  completion of this transaction
- on receiving the confirmation that the transaction has been completed, each storage provider may
  chose to remove older versions that have been superseded
- the engine may decide to get rid of stale data from its caches

This approach ensures that there's always at least one coherent game state which can be loaded.

Since storing the game state may take some time, no further persistence shall be triggered until
the previous one has been completed.

If a persistence fails to complete (timeout, database unreachable, out of disk space), a warning
shall be issued about this but the game shall continue running.
There's a chance that the problem can be fixed and that persistence can be resumed later.

## TODO

Open question: something needs to keep track of which blocks have been transferred to which client for
the following reasons:

- each block only shall be sent to each client once
- whenever a block becomes unavailable to a client (unloading) it might be re-sent to the client at
  a later point.
- whenever a block's content changes, the client that rae currently seeing this block shall receive
  an update
