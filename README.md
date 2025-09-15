# blockybuildy

Experimental blockbuilding-based search program for solving 3D and 4D Rubik's cubes

## Optimizations (to-do)

- [ ] meta search over block extensions
- [ ] don't move the same grip twice within one search
- [ ] make supercenters optional (and superridges in 4D)
- [ ] prune based on optimistic block formation heuristics
  - [principal variation](https://www.chessprogramming.org/Principal_Variation) (track which branches gave best heuristic and explore those first when deepening)
- [x] dynamically adjust goal (may be redundant with pruning)
- [ ] NISS
- [ ] insertions on inactive grips
