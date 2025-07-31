# **bevy_shatter**

Procedural glass shattering plugin for the [Bevy game engine](https://bevyengine.org/)

**Note**: This plugin uses [avian3d](https://github.com/Jondolf/avian) for collider generation, but [rapier3d](https://rapier.rs/) integration should be trivial to add in the future

# Usage

**Creating glass**

Add the Glass component to an entity. A helper is available in AutoGlass to add other needed components automatically, such as a mesh and a transform with the correct scale.

**Shattering glass**

Add the Shattered component to an entity that has Glass, and glass shards will automatically be created.

# Customizing behaviour

This plugin prioritizes user control instead of guessing what the user wants to do, at a cost of convenience for the simpler use cases. You are responsible, for example, for adding RigidBody::Dynamic to each shard of glass (if that's what you need), and you can customize the entities using hooks.

**Making the original glass entity hidden**

This plugin does not assume what you want to do with the original Glass entity. If you want it to be hidden when the glass shatters, this will have to be done manually by inserting Visibility::Hidden.

**Shards**

Are entities with the Shard component.

You can use this to, for example, make an OnAdd hook that automatically makes shards have a dynamic rigid body when added.

**Shard relationship**

Shards and their Glass are related using ShardOf and Shards. You can use this to delete all the shards belonging to a glass, make all the shards have the same material as their glass, etc.

# Examples

See the [`examples/`](https://github.com/ivsop/bevy_shatter) folder.

# Compatibility

| `bevy_shatter` | `bevy` |
| :--            | :--    |
| `0.1.0`        | `0.16` |

# How it works

Currently, the glass is broken into cells using a voronoi diagram. These cells are then extruded to 3D, creating a shard.

# Contributing

This plugin is in very early development. PRs and forks are welcome. See TODO.md for a list of things that are missing
