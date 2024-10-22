# Debug Features / QOL

- flying without freecam
- snapping rotation


# UI

- better main menu
- in game hud
  - selected block
  - currently nothing else to show


# Blocks

- good system for block properties
- top slabs, vertical slabs
- combining slabs


# Collision

- sliding off the corner of blocks sometimes will keep sliding even after the block
- can maybe still walk through walls (needs more testing)


# Performance / Optimization

- greedy meshing: combine adjacent faces with same texture into one quad
  - need to first set chunk length to 30
- remove unused block texture assets after cloning them into the global array texture
- dont clone as much data of adjacent chunks to the async task for generating a chunk mesh


# Input

- input system that allows customizing between keyboard and mouse better
- create new input types more easily


# Other

- custom schedule for update in state
- state scoped resources
