use bevy::{
    a11y::Focus,
    ecs::{entity::Entity, world::World},
    hierarchy::Parent,
};
use bevy_reactor_core::{Cx, RunContextRead, RunContextSetup, Signal};

use crate::focus::FocusVisible;

/// True if the given entity is a descendant of the given ancestor.
fn is_descendant(world: &World, e: &Entity, ancestor: &Entity) -> bool {
    let mut ha = e;
    loop {
        if ha == ancestor {
            return true;
        }
        match world.get_entity(*ha).map(|e| e.get::<Parent>()) {
            Some(Some(parent)) => ha = parent,
            _ => return false,
        }
    }
}

/// Method to create a signal that tracks whether a target entity has focus.
pub trait CreateFocusSignal {
    /// Signal that returns true when the the target has focus.
    fn create_focus_signal(&mut self, target: Entity) -> Signal<bool>;

    /// Signal that returns true when the the target, or a descendant, has focus.
    fn create_focus_within_signal(&mut self, target: Entity) -> Signal<bool>;

    /// Signal that returns true when the the target has focus and the focus ring is visible.
    fn create_focus_visible_signal(&mut self, target: Entity) -> Signal<bool>;

    /// Signal that returns true when the the target, or a descendant, has focus, and the
    /// focus ring is visible.
    fn create_focus_within_visible_signal(&mut self, target: Entity) -> Signal<bool>;
}

impl<'p, 'w> CreateFocusSignal for Cx<'p, 'w> {
    fn create_focus_signal(&mut self, target: Entity) -> Signal<bool> {
        self.create_derived(move |cx| {
            let focus = cx.use_resource::<Focus>();
            focus.0 == Some(target)
        })
    }

    fn create_focus_within_signal(&mut self, target: Entity) -> Signal<bool> {
        self.create_derived(move |cx| {
            let focus = cx.use_resource::<Focus>();
            match focus.0 {
                Some(focus) => is_descendant(cx.world(), &focus, &target),
                None => false,
            }
        })
    }

    fn create_focus_visible_signal(&mut self, target: Entity) -> Signal<bool> {
        self.create_derived(move |cx| {
            let visible = cx.use_resource::<FocusVisible>();
            let focus = cx.use_resource::<Focus>();
            visible.0 && focus.0 == Some(target)
        })
    }

    fn create_focus_within_visible_signal(&mut self, target: Entity) -> Signal<bool> {
        self.create_derived(move |cx| {
            let visible = cx.use_resource::<FocusVisible>();
            if !visible.0 {
                return false;
            }
            let focus = cx.use_resource::<Focus>();
            match focus.0 {
                Some(focus) => is_descendant(cx.world(), &focus, &target),
                None => false,
            }
        })
    }
}
