# robodoan

Experimental blockbuilding-based search program for solving 3D and 4D Rubik's cubes

## History

Named after Charles Doan, the 3^4 FMC (fewest-moves challenge) world record holder at the time this program was developed. As of September 2024, Charles Doan holds both the computer-assisted and non-computer-assisted FMC records for the 3x3x3x3 puzzle, and in fact his submission for non-computer-assisted is even shorter than the computer-assisted solution.

I'm writing this program to give the computer-assisted category some much-needed love.

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
