use std::marker::PhantomData;

use moss_hecs::{Component, DynamicBundle, Entity, EntityBuilder, Frame};
use moss_hecs_schedule::{CommandBuffer, GenericWorld};
use once_cell::sync::OnceCell;

use crate::{HierarchyMut, TreeBuilderClone};

/// Ergonomically construct trees without knowledge of frame.
///
/// This struct builds the frame using [EntityBuilder](moss_hecs::EntityBuilder)
///
/// # Example
/// ```rust
/// use moss_hecs_hierarchy::*;
/// use moss_hecs::*;
///
/// struct Tree;
/// let mut frame = Frame::default();
/// let mut builder = TreeBuilder::<Tree>::from(("root",));
/// builder.attach(("child 1",));
/// builder.attach({
///     let mut builder = TreeBuilder::new();
///     builder.add("child 2");
///     builder
/// });

/// let root = builder.spawn(&mut frame);

/// assert_eq!( frame.get::<&&'static str>(root).unwrap(), "root");

/// for (a, b) in frame
///     .descendants_depth_first::<Tree>(root)
///     .zip(["child 1", "child 2"])
/// {
///     assert_eq!( frame.get::<&&str>(a).unwrap(), b)
/// }
///
/// ```
pub struct TreeBuilder<T> {
    children: Vec<TreeBuilder<T>>,
    builder: EntityBuilder,
    marker: PhantomData<T>,
    reserved: OnceCell<Entity>,
}

impl<T: Component> TreeBuilder<T> {
    /// Construct a new empty tree
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            builder: EntityBuilder::new(),
            marker: PhantomData,
            reserved: OnceCell::new(),
        }
    }

    /// Reserve the entity which this node will spawn
    pub fn reserve(&self, frame: &impl GenericWorld) -> Entity {
        *self.reserved.get_or_init(|| frame.reserve())
    }

    /// Spawn the whole tree into the frame
    pub fn spawn(&mut self, frame: &mut Frame) -> Entity {
        let parent = self.reserve(frame);
        let builder = self.builder.build();
        frame.insert(parent, builder).unwrap();

        for mut child in self.children.drain(..) {
            let child = child.spawn(frame);
            frame.attach::<T>(child, parent).unwrap();
        }

        parent
    }

    /// Spawn the whole tree into a commandbuffer.
    /// The frame is required for reserving entities.
    pub fn spawn_deferred(&mut self, frame: &impl GenericWorld, cmd: &mut CommandBuffer) -> Entity {
        let parent = self.reserve(frame);
        let builder = self.builder.build();
        cmd.insert(parent, builder);

        for mut child in self.children.drain(..) {
            let child = child.spawn_deferred(frame, cmd);
            cmd.write(move |w: &mut Frame| {
                w.attach::<T>(child, parent).unwrap();
            });
        }
        parent
    }

    /// Add a component to the root
    pub fn add(&mut self, component: impl Component) -> &mut Self {
        self.builder.add(component);
        self
    }

    // Adds a component to all nodes
    pub fn add_all(&mut self, component: impl Component + Clone) -> &mut Self {
        for child in &mut self.children {
            child.add_all(component.clone());
        }
        self.builder.add(component);
        self
    }

    /// Add a bundle to the root
    pub fn add_bundle(&mut self, bundle: impl DynamicBundle) -> &mut Self {
        self.builder.add_bundle(bundle);
        self
    }

    /// Atttach a new subtree
    pub fn attach_tree(&mut self, child: Self) -> &mut Self {
        self.children.push(child);
        self
    }

    /// Attach a new leaf as a bundle
    pub fn attach(&mut self, child: impl Into<Self>) -> &mut Self {
        self.children.push(child.into());
        self
    }

    /// Consuming variant of [Self::attach].
    ///
    /// This is useful for nesting to alleviate the need to save an intermediate
    /// builder
    pub fn attach_move(mut self, child: impl Into<Self>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Consuming variant of [Self::attach_tree].
    /// This is useful for nesting to alleviate the need to save an intermediate
    /// builder
    pub fn attach_tree_move(mut self, child: impl Into<Self>) -> Self {
        self.children.push(child.into());
        self
    }

    /// Get a reference to the deferred tree builder's children.
    pub fn children(&self) -> &[Self] {
        self.children.as_ref()
    }

    /// Get a reference to the deferred tree builder's root.
    pub fn root(&self) -> &EntityBuilder {
        &self.builder
    }

    /// Get a mutable reference to the deferred tree builder's root.
    pub fn root_mut(&mut self) -> &mut EntityBuilder {
        &mut self.builder
    }

    /// Get a mutable reference to the tree builder's children.
    pub fn children_mut(&mut self) -> &mut Vec<TreeBuilder<T>> {
        &mut self.children
    }
}

impl<B: DynamicBundle, T: Component> From<B> for TreeBuilder<T> {
    fn from(bundle: B) -> Self {
        let mut builder = EntityBuilder::new();
        builder.add_bundle(bundle);

        Self {
            children: Vec::new(),
            builder,
            marker: PhantomData,
            reserved: OnceCell::new(),
        }
    }
}

impl<T: Component> From<TreeBuilderClone<T>> for TreeBuilder<T> {
    fn from(tree: TreeBuilderClone<T>) -> Self {
        let mut builder = EntityBuilder::new();
        builder.add_bundle(&tree.builder.build());

        let children = tree
            .children
            .into_iter()
            .map(|child| child.into())
            .collect();

        Self {
            children,
            builder,
            marker: PhantomData,
            reserved: tree.reserved,
        }
    }
}
