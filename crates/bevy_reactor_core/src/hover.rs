use bevy::{hierarchy::Parent, prelude::*};
use bevy_mod_picking::{focus::HoverMap, pointer::PointerId};

use crate::{signal::Signal, Cx, RunContextRead, RunContextSetup};

/// Component which tracks whether the pointer is hovering over an entity.
#[derive(Default, Component)]
pub struct Hovering(pub bool);

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
