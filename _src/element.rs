use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    effect_target::{EffectTarget, EntityEffect},
    node_span::NodeSpan,
    parent_view::{ChildView, ParentView},
    view::View,
    DespawnScopes, IntoView, TrackingScope, ViewRef,
};

/// A basic UI element
#[derive(Default)]
pub struct Element<B: Bundle + Default>
where
    Self: EffectTarget + ParentView,
{
    /// Debug name for this element.
    debug_name: String,

    /// The visible UI node for this element.
    display: Option<Entity>,

    /// Children of this element.
    children: Vec<ChildView>,

    /// List of effects to be added to the element.
    effects: Vec<Box<dyn EntityEffect>>,

    marker: PhantomData<B>,
}

impl<B: Bundle + Default> Element<B> {
    /// Construct a new `Element`.
    pub fn new() -> Self {
        Self {
            debug_name: String::new(),
            display: None,
            children: Vec::new(),
            effects: Vec::new(),
            marker: PhantomData,
        }
    }

    /// Construct a new `Element` with a given entity id.
    pub fn for_entity(node: Entity) -> Self {
        Self {
            debug_name: String::new(),
            display: Some(node),
            children: Vec::new(),
            effects: Vec::new(),
            marker: PhantomData,
        }
    }

    /// Set the debug name for this element.
    pub fn named(mut self, name: &str) -> Self {
        self.debug_name = name.to_string();
        self
    }

    // pub fn insert_computed_ref<
    //     T: Component,
    //     F1: Send + Sync + 'static + FnMut() -> T,
    //     F2: Send + Sync + 'static + FnMut(&mut Re, &mut T),
    // >(
    //     mut self,
    //     init: F1,
    //     update: F2,
    // ) -> Self {
    //     self.producers.push(Arc::new(Mutex::new(BundleComputedRef {
    //         target: None,
    //         init,
    //         update,
    //         tracker: None,
    //         marker: PhantomData,
    //     })));
    //     self
    // }

    /// Attach the children to the node. Note that each child view may produce multiple nodes,
    /// or none.
    fn attach_children(&self, world: &mut World) {
        let flat = self.child_entities();
        world
            .entity_mut(self.display.unwrap())
            .replace_children(&flat);
    }
}

impl<B: Bundle + Default> EffectTarget for Element<B> {
    fn add_effect(&mut self, effect: Box<dyn EntityEffect>) {
        self.effects.push(effect);
    }
}

impl<B: Bundle + Default> ParentView for Element<B> {
    fn get_children(&self) -> &Vec<ChildView> {
        self.children.as_ref()
    }

    fn get_children_mut(&mut self) -> &mut Vec<ChildView> {
        self.children.as_mut()
    }
}

impl<B: Bundle + Default> View for Element<B> {
    fn nodes(&self) -> NodeSpan {
        match self.display {
            None => NodeSpan::Empty,
            Some(node) => NodeSpan::Node(node),
        }
    }

    fn build(&mut self, view_entity: Entity, world: &mut World) {
        if self.debug_name.is_empty() {
            world.entity_mut(view_entity).insert(Name::new("Element"));
        } else {
            world
                .entity_mut(view_entity)
                .insert(Name::new(format!("Element::{}", self.debug_name)));
        }

        // Build display entity if it doesn't already exist.
        let display = match self.display {
            Some(display) => {
                world
                    .entity_mut(display)
                    .insert((B::default(), Name::new(self.debug_name.clone())));
                display
            }
            None => {
                let entity = world
                    .spawn((B::default(), Name::new(self.debug_name.clone())))
                    .id();
                self.display = Some(entity);
                entity
            }
        };

        // Insert components from effects.
        if !self.effects.is_empty() {
            let mut tracking = TrackingScope::new(world.change_tick());
            for effect in self.effects.iter_mut() {
                effect.start(view_entity, display, world, &mut tracking);
            }
            world.entity_mut(view_entity).insert(tracking);
        }

        // Build child nodes.
        for child in self.children.iter_mut() {
            child.entity = Some(ViewRef::spawn(&child.view, view_entity, world));
        }

        self.attach_children(world);
    }

    fn raze(&mut self, view_entity: Entity, world: &mut World) {
        assert!(self.display.is_some());
        self.raze_children(world);

        // Delete the display node.
        world.entity_mut(self.display.unwrap()).remove_parent();
        world.entity_mut(self.display.unwrap()).despawn();
        self.display = None;

        // Delete all reactions.
        world.despawn_owned_recursive(view_entity);
    }

    fn children_changed(&mut self, _view_entity: Entity, world: &mut World) -> bool {
        self.attach_children(world);
        true
    }
}

impl<B: Bundle + Default> IntoView for Element<B> {
    fn into_view(self) -> ViewRef {
        ViewRef::new(self)
    }
}
