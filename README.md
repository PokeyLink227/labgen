
## Feature Compatability

|  Maze Type | Wrapping | Exclusions | Rooms |
|------------|----------|------------|-------|
| backtrack    | ✅ | ✅ | ✅ |
| prim         | ✅ | ✅ | ✅ |
| binary-tree  | ❌ | ✅ | ❌ |
| sidewinder   | ❌ | ❌ | ❌ |
| noise        | ❌ | ❌ | ❌ |
| growing-tree | ✅ | ✅ | ✅ |
| wilsons      | ✅ | ✅ | ✅ |
| kruskal      | ✅ | ✅ | ✅ |

|  Image Type | Animation | Batching | Wrapping | Exclusions | Rooms |
|-------------|-----------|----------|----------|------------|-------|
| png            | ❌ | ❌ | ✅ | ✅ | ✅ |
| gif            | ✅ | ✅ | ✅ | ✅ | ❌ |
| gif-compressed | ✅ | ❌ | ✅ | ✅ | ✅ |

### Warnings

- None (for now)

## Feature descriptions
- Exclusions
  - areas of the maze that will not be filled and are thus inaccessible
- Rooms
  - connected areas of the maze that by default will not disrupt the perfect aspect of the maze
  - if doors are placed manually then this is no longer guaranteed
- Regions
  - areas of the maze that are filled with connected maze
  - (WIP) can be overridden to form disconnected but adjacent parts of the maze without the use of exclusions
