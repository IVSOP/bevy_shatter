# API limitations

- [ ] Allow optionally spawning the shards at the same time as the glass. The behaviour of Shattered will change, and the user needs to be able to have another hook for this
- [ ] Do not assume the material is `MeshMaterial3d(Handle<StandardMaterial>)`
- [ ] Use other approaches other than just voronoi diagrams, since it makes shards too poligonal and not triangular
- [ ] Every shard's mesh being different means instancing is not possible. Can this be improved?
- [ ] Rapier integration

# Issues

- [ ] Glass can blink for a couple of frames, where it has already become invisible but the shards are not yet visible
- [ ] use bevy_rand instead of fastrand, or make it clear how to compile to wasm
