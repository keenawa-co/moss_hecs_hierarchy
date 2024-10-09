# moss_hecs_hierarchy

[![Cargo](https://img.shields.io/crates/v/hecs-hierarchy.svg)](https://crates.io/crates/hecs-hierarchy)
[![Documentation](https://docs.rs/hecs-hierarchy/badge.svg)](https://docs.rs/hecs-hierarchy)
[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)

Hierarchy implementation for use with the _hecs_ ECS.

### Features

- [x] Iterate children of parent
- [x] Lookup parent of child
- [x] Traverse hierarchy depth first
- [x] Traverse hierarchy breadth first
- [x] Traverse ancestors
- [x] Detach child from hierarchy
- [x] Ergonomic tree building
- [ ] Reverse iteration
- [ ] Sorting
- [ ] (Optional) associated data to relation

### Motivation

An ECS is a fantastic design principle for designing software which allows a
data oriented design. Most of the time, the ECS is flat with maybe a few
components referencing each other via `Entity` ids. Sometimes however, the need
to create and manage proper, well behaved graphs, arises.

This is were hecs-hierarchy comes in and gives the ability to manage directed
graphs that can connect entities. This is very useful when developing a UI
library using the ECS design pattern, or purely for grouping entities together
from the same model.

### Usage

Import the [Hierarchy](crate::Hierarchy) trait which extends [hecs::World](hecs::World)

The trait [Hierarchy](crate::Hierarchy) extends [hecs::World](hecs::World) with functions for
manipulating and iterating the hierarchy tree.

The hierarchy uses a marker type which makes it possible for a single entity to belong to
several hierarchy trees.

See the [documentation](https://docs.rs/hecs-hierarchy), more specifically the
[Hierarchy](https://docs.rs/hecs-hierarchy/0.1.7/hecs_hierarchy/trait.Hierarchy.html)
trait

Example usage:

```rust
use moss_hecs_hierarchy::*;

// Marker type which allows several hierarchies.
struct Tree;

let mut frame = moss_hecs::Frame::default();

// Create a root entity, there can be several.
let root = frame.spawn(("Root",));

// Create a loose entity
let child = frame.spawn(("Child 1",));

// Attaches the child to a parent, in this case `root`
frame.attach::<Tree>(child, root).unwrap();

// Iterate children
for child in frame.children::<Tree>(root) {
    let name = frame.get::<&&str>(child).unwrap();
    println!("Child: {:?} {}", child, *name);
}

// Add a grandchild
frame.attach_new::<Tree, _>(child, ("Grandchild",)).unwrap();

// Iterate recursively
for child in frame.descendants_depth_first::<Tree>(root) {
    let name = frame.get::<&&str>(child).unwrap();
    println!("Child: {:?} {}", child, *name)
}

// Detach `child` and `grandchild`
frame.detach::<Tree>(child).unwrap();

let child2 = frame.attach_new::<Tree, _>(root, ("Child 2",)).unwrap();

// Reattach as a child of `child2`
frame.attach::<Tree>(child, child2).unwrap();

frame.attach_new::<Tree, _>(root, ("Child 3",)).unwrap();

// Hierarchy now looks like this:
// Root
// |-------- Child 3
// |-------- Child 2
//           |-------- Child 1
//                     |-------- Grandchild

```

### Inspiration

This project is heavily inspired by `Shipyard`'s hierarchy implementation and
exposes a similar API.

- [shipyard-hierarchy](https://github.com/dakom/shipyard-hierarchy)
