# blockybuildy

Experimental blockbuilding-based search program for solving 3D and 4D Rubik's cubes

## Optimizations (to-do)

- [ ] search multiple routes at once / meta search over block extensions
- [ ] consider other representations of rotations that can use smaller LUTs
- [x] don't move the same grip twice within one search
- [ ] make supercenters optional (and superridges in 4D)
- [x] prune based on optimistic block formation heuristics
- [x] dynamically adjust goal (may be redundant with pruning)
- [ ] NISS
- [ ] insertions on inactive grips
- [ ] GPGPU???
