
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
| kruskal | ✅ | ✅ | ❌ |

### Warnings

- wilson algorithm will run forever if some nodes are inaccessible

## Feature descriptions
- Exclusions
  - areas of the maze that will not be filled and are thus inaccessible
- Rooms
  - connected areas of the maze that by default will not disrupt the perfect aspect of the maze
  - if doors are placed manually then this is no longer guaranteed
