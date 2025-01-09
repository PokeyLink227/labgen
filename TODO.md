(maze)
modify algos to all floodfill
add room generation and passage uncarving
add rooms and empty zones
find each disconnected region and fill each with a maze
add regions that get connected at the end only?
add ellers and growing forest - for sure second one
improve history possibly with an enum to support
    - temp cells in maze
    - uncarving cells
add option to wrap around edges
modify directions to support corners
    remove need to manually select direction 1-4
3d mazes (would need gui to make effectively maybe bevy + egui?) (or egui + wgpu)


instead of flood fill
carve through maze like normal and when crossing the boundry into a new area swap algo
- need to keep track of stack for backtracker accross all regions (stack is shared)
- need to keep track of open list for prim (shared bewteen regions)

each floodfill function will only return under 2 conditions
1. a boundry was crossed
2. space is full

so always swap algo on return and additionally keep track of total nodes visited
