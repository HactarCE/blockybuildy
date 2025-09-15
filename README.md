# blockybuildy

Experimental blockbuilding-based search program for solving 3D and 4D Rubik's cubes

## Optimizations

- meta search over block extensions
- don't move the same grip twice within one search
- prune based on optimistic block formation heuristics
  - [principal variation](https://www.chessprogramming.org/Principal_Variation) (track which branches gave best heuristic and explore those first when deepening)
- dynamically adjust goal (may be redundant with pruning)
- insertions on inactive grips
