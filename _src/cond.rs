use bevy::ecs::world::World;
use bevy::prelude::*;

use crate::node_span::NodeSpan;
use crate::{DespawnScopes, DisplayNodeChanged, IntoView, Rcx, TrackingScope, View, ViewRef};

pub enum CondState {
    Unset,
    True((ViewRef, Entity)),
    False((ViewRef, Entity)),
}

/// A conditional view which renders one of two children depending on the condition expression.
pub struct Cond<Test: 'static, Pos: IntoView, PosFn: Fn() -> Pos, Neg: IntoView, NegFn: Fn() -> Neg>
{
    test: Test,
    pos: PosFn,
    neg: NegFn,
    state: CondState,
}

impl<
        Test: Fn(&Rcx) -> bool,
        Pos: IntoView,
        PosFn: Fn() -> Pos,
        Neg: IntoView,
        NegFn: Fn() -> Neg,
    > Cond<Test, Pos, PosFn, Neg, NegFn>
{
    /// Construct a new conditional View.
    pub fn new(test: Test, pos: PosFn, neg: NegFn) -> Self {
        Self {
            test,
            pos,
            neg,
            state: CondState::Unset,
        }
    }

    fn build_branch_state<V: IntoView, Factory: Fn() -> V>(
        &self,
        branch: &Factory,
        parent: Entity,
        world: &mut World,
    ) -> (ViewRef, Entity) {
        let state_view = (branch)().into_view();
        let state_entity = ViewRef::spawn(&state_view, parent, world);
        // assert!(
        //     world.entity_mut(parent).get::<Parent>().is_some(),
        //     "Cond should have a parent view"
        // );
        (state_view, state_entity)
    }
}

impl<
        Test: Fn(&Rcx) -> bool,
        Pos: IntoView,
        PosFn: Fn() -> Pos,
        Neg: IntoView,
        NegFn: Fn() -> Neg,
    > View for Cond<Test, Pos, PosFn, Neg, NegFn>
{
    fn nodes(&self) -> NodeSpan {
        match self.state {
            CondState::Unset => NodeSpan::Empty,
            CondState::True(ref true_state) => true_state.0.nodes(),
            CondState::False(ref false_state) => false_state.0.nodes(),
        }
    }

    fn build(&mut self, view_entity: Entity, world: &mut World) {
        world.entity_mut(view_entity).insert(Name::new("Cond"));
        let mut tracking = TrackingScope::new(world.change_tick());
        self.react(view_entity, world, &mut tracking);
        world.entity_mut(view_entity).insert(tracking);
        assert!(
            world.entity_mut(view_entity).get::<Parent>().is_some(),
            "Cond should have a parent view"
        );
    }

    fn react(&mut self, view_entity: Entity, world: &mut World, tracking: &mut TrackingScope) {
        let re = Rcx::new(world, view_entity, tracking);
        let cond = (self.test)(&re);
        // possibly raze previous state
        match self.state {
            CondState::True(_) if cond => {
                return;
            }
            CondState::False(_) if !cond => {
                return;
            }
            CondState::True((ref mut true_state, entity)) => {
                true_state.raze(entity, world);
            }
            CondState::False((ref mut false_state, entity)) => {
                false_state.raze(entity, world);
            }
            _ => {}
        }

        self.state = if cond {
            CondState::True(self.build_branch_state(&self.pos, view_entity, world))
        } else {
            CondState::False(self.build_branch_state(&self.neg, view_entity, world))
        };

        world.entity_mut(view_entity).insert(DisplayNodeChanged);
    }

    fn raze(&mut self, view_entity: Entity, world: &mut World) {
        match self.state {
            CondState::True((ref mut true_state, entity)) => true_state.raze(entity, world),
            CondState::False((ref mut false_state, entity)) => false_state.raze(entity, world),
            CondState::Unset => {}
        }
        self.state = CondState::Unset;
        world.despawn_owned_recursive(view_entity);
    }
}

impl<
        Test: Send + Sync + Fn(&Rcx) -> bool,
        Pos: 'static + IntoView,
        PosFn: Send + Sync + 'static + Fn() -> Pos,
        Neg: 'static + IntoView,
        NegFn: Send + Sync + 'static + Fn() -> Neg,
    > IntoView for Cond<Test, Pos, PosFn, Neg, NegFn>
{
    fn into_view(self) -> ViewRef {
        ViewRef::new(self)
    }
}
