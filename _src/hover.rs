use bevy::{hierarchy::Parent, prelude::*};
use bevy_mod_picking::{focus::HoverMap, pointer::PointerId};

use crate::{signal::Signal, Cx, RunContextRead, RunContextSetup};

/// Component which tracks whether the pointer is hovering over an entity.
#[derive(Default, Component)]
pub(crate) struct Hovering(pub bool);

// Note: previously this was implemented as a Reaction, however it was reacting every frame
// because HoverMap is mutated every frame regardless of whether or not it changed.
pub(crate) fn update_hover_states(
    hover_map: Option<Res<HoverMap>>,
    mut hovers: Query<(Entity, &mut Hovering)>,
    parent_query: Query<&Parent>,
) {
    let Some(hover_map) = hover_map else { return };
    let hover_set = hover_map.get(&PointerId::Mouse);
    for (entity, mut hoverable) in hovers.iter_mut() {
        let is_hovering = match hover_set {
            Some(map) => map
                .iter()
                .any(|(ha, _)| parent_query.iter_ancestors(*ha).any(|e| e == entity)),
            None => false,
        };
        if hoverable.0 != is_hovering {
            hoverable.0 = is_hovering;
        }
    }
}

/// Method to create a signal that tracks whether the mouse is hovering over the given entity.
pub trait CreateHoverSignal {
    /// Signal that returns true when the mouse is hovering over the given entity or a descendant.
    fn create_hover_signal(&mut self, target: Entity) -> Signal<bool>;
}

impl<'p, 'w> CreateHoverSignal for Cx<'p, 'w> {
    fn create_hover_signal(&mut self, target: Entity) -> Signal<bool> {
        self.world_mut().entity_mut(target).insert(Hovering(false));
        let hovering = self.create_derived(move |cx| {
            cx.use_component::<Hovering>(target)
                .map(|h| h.0)
                .unwrap_or(false)
        });
        hovering
    }
}
