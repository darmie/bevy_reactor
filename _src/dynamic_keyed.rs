use bevy::core::Name;
use bevy::ecs::entity::Entity;
use bevy::ecs::world::World;

use crate::{Cx, DespawnScopes, DisplayNodeChanged, TrackingScope, ViewRef};
use crate::{IntoView, View};

use crate::node_span::NodeSpan;

/// A dynamic view which computes a value reactively, then (non-reactively) constructs
/// a [`View`] from that value. This is useful in cases where we want to control precisely
/// which dependencies we are reacting to.
#[doc(hidden)]
pub struct DynamicKeyed<Key, KeyFn: Fn(&mut Cx) -> Key, V: IntoView, F: Fn(Key) -> V + Send> {
    state: Option<(ViewRef, Entity)>,
    key: KeyFn,
    factory: F,
}

impl<Key, KeyFn: Fn(&mut Cx) -> Key, V: IntoView, F: Fn(Key) -> V + Send>
    DynamicKeyed<Key, KeyFn, V, F>
{
    /// Construct a new `DynamicKeyed` view.
    ///
    /// # Arguments
    /// * key - A function which reactively computes a value which will be passed to the
    ///   factory function.
    /// * factory - A function which non-reactively constructs a view from the key value.
    pub fn new(key: KeyFn, factory: F) -> Self {
        Self {
            state: None,
            key,
            factory,
        }
    }
}

impl<Key, KeyFn: Fn(&mut Cx) -> Key, V: IntoView, F: Fn(Key) -> V + Send> View
    for DynamicKeyed<Key, KeyFn, V, F>
{
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
            .insert((tracking, Name::new("DynamicKeyed")));
    }

    fn react(&mut self, view_entity: Entity, world: &mut World, tracking: &mut TrackingScope) {
        if let Some((view, entity)) = self.state.take() {
            view.raze(entity, world);
        }

        let key = (self.key)(&mut Cx::new(world, view_entity, tracking));
        let view = (self.factory)(key).into_view();
        let entity = ViewRef::spawn(&view, view_entity, world);
        world.entity_mut(view_entity).insert(DisplayNodeChanged);
        self.state = Some((view, entity));
    }

    fn raze(&mut self, view_entity: Entity, world: &mut World) {
        if let Some((ref view, entity)) = self.state {
            view.raze(entity, world)
        }
        self.state = None;
        world.despawn_owned_recursive(view_entity);
    }
}

impl<
        Key: 'static,
        KeyFn: Send + Sync + 'static + Fn(&mut Cx) -> Key,
        V: IntoView + 'static,
        F: Send + Sync + 'static + Fn(Key) -> V,
    > IntoView for DynamicKeyed<Key, KeyFn, V, F>
{
    fn into_view(self) -> ViewRef {
        ViewRef::new(self)
    }
}
