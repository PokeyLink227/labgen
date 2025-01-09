

instead of flood fill
carve through maze like normal and when crossing the boundry into a new area swap algo
- need to keep track of stack for backtracker accross all regions (stack is shared)
- need to keep track of open list for prim (shared bewteen regions)

each floodfill function will only return under 2 conditions
1. a boundry was crossed
2. space is full

so always swap algo on return and additionally keep track of total nodes visited


## Feature Compatability

|  Maze Type | Wrapping | Exclusions | Rooms |
|------------|----------|------------|-------|
| backtrack | ✅ | ✅  | ❌ |
| prim | ✅ | ✅ | ❌ |
| binary-tree | ❌ | ✅ | ❌|
| sidewinder | ❌ | ❌ | ❌ |
| noise | ❌ | ❌ | ❌ |
| growing-tree | ✅ | ✅ | ❌ |
| wilsons | ✅ | ✅ | ❌ |
| kruskal | ❌ | ❌ | ❌ |
