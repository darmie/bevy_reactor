use bevy::core::Name;
use bevy::ecs::entity::Entity;
use bevy::ecs::world::World;

use crate::{Cx, DespawnScopes, DisplayNodeChanged, TrackingScope, ViewRef};
use crate::{IntoView, View};

use crate::node_span::NodeSpan;

/// A dynamic view which can change its content based on a function. The inner view is razed
/// and rebuilt whenever the function reacts.
pub struct Dynamic<V: IntoView, F: Fn(&mut Cx) -> V + Send> {
    state: Option<(ViewRef, Entity)>,
    factory: F,
}

impl<V: IntoView, F: Fn(&mut Cx) -> V + Send> Dynamic<V, F> {
    /// Construct a new dynamic view.
    pub fn new(factory: F) -> Self {
        Self {
            state: None,
            factory,
        }
    }
}

impl<V: IntoView, F: Fn(&mut Cx) -> V + Send> View for Dynamic<V, F> {
    fn nodes(&self) -> NodeSpan {
        match self.state {
            None => NodeSpan::Empty,
            Some((ref view, _)) => view.nodes(),
        }
    }

    fn build(&mut self, view_entity: bevy::prelude::Entity, world: &mut World) {
        let mut tracking = TrackingScope::new(world.change_tick());
        self.react(view_entity, world, &mut tracking);
        world
            .entity_mut(view_entity)
            .insert((tracking, Name::new("Dynamic")));
    }

    fn react(&mut self, view_entity: Entity, world: &mut World, tracking: &mut TrackingScope) {
        if let Some((view, entity)) = self.state.take() {
            view.raze(entity, world);
        }

        let view = (self.factory)(&mut Cx::new(world, view_entity, tracking)).into_view();
        let entity = ViewRef::spawn(&view, view_entity, world);
        self.state = Some((view, entity));
        world.entity_mut(view_entity).insert(DisplayNodeChanged);
    }

    fn raze(&mut self, view_entity: Entity, world: &mut World) {
        if let Some((ref view, entity)) = self.state {
            view.raze(entity, world)
        }
        self.state = None;
        world.despawn_owned_recursive(view_entity);
    }
}

impl<V: IntoView + 'static, F: Fn(&mut Cx) -> V + Sync + Send + 'static> IntoView
    for Dynamic<V, F>
{
    fn into_view(self) -> ViewRef {
        ViewRef::new(self)
    }
}

// // Commented out because it conflicts with the ViewTemplate -> ViewRef conversion.
// impl<V: IntoView + 'static, F: Fn(&Rcx) -> V + Sync + Send + 'static> IntoView for F {
//     fn into_view(self) -> Self {
//         ViewRef::new(Dynamic::new(self))
//     }
// }
