# API limitations / missing features

- [ ] Allow optionally spawning the shards at the same time as the glass. The behaviour of Shattered will change, and the user needs to be able to have another hook for this
- [ ] Do not assume the material is `MeshMaterial3d(Handle<StandardMaterial>)`
- [ ] Use other approaches other than just voronoi diagrams, since it makes shards too poligonal and not triangular. Also could add more realistic shatter, shattering into triangles, or at least generating the cell points in a distribution that is tighter near the impact point
- [ ] Every shard's mesh being different means instancing is not possible. Can this be improved?
- [ ] Rapier integration
- [ ] Allow LOD, where less cells are used if the glass is far away
- [ ] The hooks I use in the character example will probably be very common, should be available in the lib for convenience

# Issues

- [ ] Glass can blink for a couple of frames, where it has already become invisible but the shards are not yet visible
- [ ] In the click example, spectator plugin hides the mouse even when egui is clicked. They have a PR for this but it is old and was never merged

# Bevy integration

- [ ] Use bevy_rand instead of fastrand, or make it clear how to compile to wasm (the wasm feature)
- [ ] Bevy's [extrusion](https://docs.rs/bevy/latest/bevy/render/mesh/trait.Extrudable.html) used to not cover my use case, see if it works now
- [ ] Use bevy picking in the click example??

# Examples

- [ ] click: click_shatter should use a filter, see the comment
- [ ] more comments on functions
- [ ] add physics drawing to the character example

# Math is hard

- [ ] maybe project_to_glass functions could already return the Vec2 relative position

# Other

See the TODO:, FIX:, etc  that are spread throughout the code
