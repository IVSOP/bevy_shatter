# API limitations

- [ ] Allow optionally spawning the shards at the same time as the glass. The behaviour of Shattered will change, and the user needs to be able to have another hook for this
- [ ] Do not assume the material is `MeshMaterial3d(Handle<StandardMaterial>)`
- [ ] Use other approaches other than just voronoi diagrams, since it makes shards too poligonal and not triangular. Also could add more realistic shatter, shattering into triangles, or at least generating the cell points in a distribution that is tighter near the impact point
- [ ] Every shard's mesh being different means instancing is not possible. Can this be improved?
- [ ] Rapier integration
- [ ] Allow LOD, where less cells are used if the glass is far away

# Issues

- [ ] Glass can blink for a couple of frames, where it has already become invisible but the shards are not yet visible
- [ ] use bevy_rand instead of fastrand, or make it clear how to compile to wasm
