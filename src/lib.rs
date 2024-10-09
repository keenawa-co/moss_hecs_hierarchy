//! # moss_hecs_hierarchy
//!
//! [![Cargo](https://img.shields.io/crates/v/hecs-hierarchy.svg)](https://crates.io/crates/hecs-hierarchy)
//! [![Documentation](https://docs.rs/hecs-hierarchy/badge.svg)](https://docs.rs/hecs-hierarchy)
//! [![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)
//!
//! Hierarchy implementation for moss_hecs ECS.
//!
//! ## Features
//! - [X] Iterate children of parent
//! - [X] Lookup parent of child
//! - [X] Traverse hierarchy depth first
//! - [X] Traverse hierarchy breadth first
//! - [X] Traverse ancestors
//! - [X] Detach child from hierarchy
//! - [ ] Reverse iteration
//! - [ ] Sorting
//! - [ ] (Optional) associated data to relation
//!
//! ## Getting Started
//!
//! Include both `moos_` and `moss_hecs_hierarchy` as dependencies in your `Cargo.toml`.
//!
//! `moss_hierarchy` does not re-export `moss_hecs`
//!
//! ```toml
//! [dependencies]
//! moss_hecs = 0.10
//! moss_hecshierarchy = 0.12
//! ```
//!
//! ## Motivation
//!
//! An ECS is a fantastic design principle for designing software which allows a
//! data oriented design. Most of the time, the ECS is flat with maybe a few
//! components referencing each other via `Entity` ids.  Sometimes however, the need
//! to create and manage proper, well behaved graphs, arises.
//!
//! This is were hecs-hierarchy comes in and gives the ability to manage directed
//! graphs that can connect entities. This is very useful when developing a UI
//! library using the ECS design pattern, or purely for grouping entities together
//! from the same model.
//!
//! ## Usage
//!
//! Import the [Hierarchy](crate::Hierarchy) trait which extends [hecs::World](hecs::World)
//!
//! The trait [Hierarchy](crate::Hierarchy) extends [hecs::World](hecs::World) with functions for
//! manipulating and iterating the hierarchy tree.
//!
//! The hierarchy uses a marker type which makes it possible for a single entity to belong to
//! several hierarchy trees.
//!
//! See the [documentation](https://docs.rs/hecs-hierarchy), more specifically the
//! [Hierarchy](https://docs.rs/hecs-hierarchy/0.1.7/hecs_hierarchy/trait.Hierarchy.html)
//! trait
//!
//! Example usage:
//! ```
//! use moss_hecs_hierarchy::*;
//!
//! // Marker type which allows several hierarchies.
//! struct Tree;
//!
//! let mut frame = moss_hecs::Frame::default();
//!
//! // Create a root entity, there can be several.
//! let root = frame.spawn(("Root",));
//!
//! // Create a loose entity
//! let child = frame.spawn(("Child 1",));
//!
//! // Attaches the child to a parent, in this case `root`
//! frame.attach::<Tree>(child, root).unwrap();
//!
//! // Iterate children
//! println!("Iterating children:");
//! for child in frame.children::<Tree>(root) {
//!     let name = frame.get::<&&str>(child).unwrap();
//!     println!("  Child: {:?} {}", child, *name);
//! }
//!
//! // Add a grandchild
//! frame.attach_new::<Tree, _>(child, ("Grandchild",)).unwrap();
//!
//! // Iterate recursively
//! println!("Iterating descendants recursively:");
//! for descendant in frame.descendants_depth_first::<Tree>(root) {
//!     let name = frame.get::<&&str>(descendant).unwrap();
//!     println!("  Descendant: {:?} {}", descendant, *name)
//! }
//!
//! // Detach `child` and `grandchild`
//! frame.detach::<Tree>(child).unwrap();
//!
//! let child2 = frame.attach_new::<Tree, _>(root, ("Child 2",)).unwrap();
//!
//! // Reattach as a child of `child2`
//! frame.attach::<Tree>(child, child2).unwrap();
//!
//! frame.attach_new::<Tree, _>(root, ("Child 3",)).unwrap();
//!
//! // Hierarchy now looks like this:
//! // Root
//! // |-------- Child 3
//! // |-------- Child 2
//! //           |-------- Child 1
//! //                     |-------- Grandchild
//!
//! // Iterate recursively
//! println!("Iterating descendants recursively:");
//! for descendant in frame.descendants_depth_first::<Tree>(root) {
//!     let name = frame.get::<&&str>(descendant).unwrap();
//!     println!("  Descendant: {:?} {}", descendant, *name)
//! }
//!
//! ```
//!
//! ## Inspiration
//!
//! This project is heavily inspired by `Shipyard`'s hierarchy implementation and
//! exposes a similar API.
//!
//! - [shipyard-hierarchy](https://github.com/dakom/shipyard-hierarchy)

mod builder;
mod builder_clone;
mod components;
mod hierarchy;
mod iter;

pub use builder::*;
pub use builder_clone::*;
pub use components::*;
pub use hierarchy::*;
pub use iter::*;

pub use moss_hecs_schedule::Error;
