(maze)
add SVG and mp4 support
add regions that get connected at the end only?
add ellers and growing forest - for sure second one
sparse mazes
2.5d mazes
improve history possibly with an enum to support
    - temp cells in maze
    - backtracking
3d mazes (would need gui to make effectively maybe bevy + egui?) (or egui + wgpu) (iced)
wrapping rooms
user-defined textures for cell-types
pass algo a function to use for finding adjacant tiles

make string parsing use regex.captures to allow users to use reserved characters


instead of flood fill
carve through maze like normal and when crossing the boundry into a new area swap algo
- need to keep track of stack for backtracker accross all regions (stack is shared)
- need to keep track of open list for prim (shared bewteen regions)

each floodfill function will only return under 2 conditions
1. a boundry was crossed
2. space is full

so always swap algo on return and additionally keep track of total nodes visited
