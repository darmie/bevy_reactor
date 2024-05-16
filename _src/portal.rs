use bevy::prelude::*;

use crate::{node_span::NodeSpan, view::View, DespawnScopes, IntoView, ViewRef};

/// A `Portal` represents a view that is displayed with no parent, causing it's location to
/// be relative to the window rather than any parent view.
pub struct Portal {
    view: ViewRef,
    entity: Option<Entity>,
}

impl Portal {
    /// Construct a new `Portal`.
    pub fn new(view: impl IntoView) -> Self {
        Self {
            view: view.into_view(),
            entity: None,
        }
    }
}

impl View for Portal {
    fn nodes(&self) -> NodeSpan {
        NodeSpan::Empty
    }

    fn build(&mut self, view_entity: Entity, world: &mut World) {
        assert!(self.entity.is_none());
        world.entity_mut(view_entity).insert(Name::new("Portal"));
        self.entity = Some(ViewRef::spawn(&self.view, view_entity, world));
    }

    fn raze(&mut self, view_entity: Entity, world: &mut World) {
        self.view.raze(self.entity.unwrap(), world);
        self.entity = None;
        world.despawn_owned_recursive(view_entity);
    }

    fn children_changed(&mut self, _view_entity: Entity, _world: &mut World) -> bool {
        true
    }
}

impl IntoView for Portal {
    fn into_view(self) -> ViewRef {
        ViewRef::new(self)
    }
}
