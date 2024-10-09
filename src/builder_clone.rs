use std::marker::PhantomData;

use moss_hecs::{Component, DynamicBundleClone, Entity, EntityBuilderClone, Frame};
use moss_hecs_schedule::{CommandBuffer, GenericWorld};
use once_cell::sync::OnceCell;

use crate::HierarchyMut;

/// Cloneable version of the [crate::TreeBuilder]
pub struct TreeBuilderClone<T> {
    pub(crate) children: Vec<TreeBuilderClone<T>>,
    pub(crate) builder: EntityBuilderClone,
    pub(crate) marker: PhantomData<T>,
    pub(crate) reserved: OnceCell<Entity>,
}

impl<T: Component> TreeBuilderClone<T> {
    /// Construct a new empty tree
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            builder: EntityBuilderClone::new(),
            marker: PhantomData,
            reserved: OnceCell::new(),
        }
    }

    /// Reserve the entity which this node will spawn
    pub fn reserve(&self, frame: &impl GenericWorld) -> Entity {
        *self.reserved.get_or_init(|| frame.reserve())
    }

    /// Spawn the whole tree into the world
    pub fn spawn(self, frame: &mut Frame) -> Entity {
        let parent = self.reserve(frame);
        let builder = self.builder.build();
        frame.insert(parent, &builder).unwrap();

        for child in self.children {
            let child = child.spawn(frame);
            frame.attach::<T>(child, parent).unwrap();
        }

        parent
    }

    /// Spawn the whole tree into a commandbuffer.
    /// The world is required for reserving entities.
    pub fn spawn_deferred(self, frame: &impl GenericWorld, cmd: &mut CommandBuffer) -> Entity {
        let parent = self.reserve(frame);
        let builder = self.builder.build();
        cmd.insert(parent, &builder);

        for child in self.children {
            let child = child.spawn_deferred(frame, cmd);
            cmd.write(move |w: &mut Frame| {
                w.attach::<T>(child, parent).unwrap();
            });
        }
        parent
    }

    /// Add a component to the root
    pub fn add(&mut self, component: impl Component + Clone) -> &mut Self {
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
    pub fn add_bundle(&mut self, bundle: impl DynamicBundleClone) -> &mut Self {
        self.builder.add_bundle(bundle);
        self
    }

    /// Atttach a new subtree
    pub fn attach_tree(&mut self, child: Self) -> &mut Self {
        self.children.push(child);
        self
    }

    /// Attach a new leaf
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
    pub fn root(&self) -> &EntityBuilderClone {
        &self.builder
    }

    /// Get a mutable reference to the deferred tree builder's builder.
    pub fn root_mut(&mut self) -> &mut EntityBuilderClone {
        &mut self.builder
    }

    /// Get a mutable reference to the tree builder clone's children.
    pub fn children_mut(&mut self) -> &mut Vec<TreeBuilderClone<T>> {
        &mut self.children
    }
}

impl<T> Clone for TreeBuilderClone<T> {
    fn clone(&self) -> Self {
        Self {
            children: self.children.clone(),
            builder: self.builder.clone(),
            marker: PhantomData,
            reserved: OnceCell::new(),
        }
    }
}

impl<B: DynamicBundleClone, T: Component> From<B> for TreeBuilderClone<T> {
    fn from(bundle: B) -> Self {
        let mut builder = EntityBuilderClone::new();
        builder.add_bundle(bundle);

        Self {
            children: Vec::new(),
            builder,
            marker: PhantomData,
            reserved: OnceCell::new(),
        }
    }
}
