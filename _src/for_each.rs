use std::ops::Range;

use bevy::core::Name;
use bevy::ecs::entity::Entity;
use bevy::ecs::world::World;
use bevy::hierarchy::Parent;

use crate::{lcs::lcs, View};
use crate::{DespawnScopes, DisplayNodeChanged, IntoView, Rcx, TrackingScope, ViewRef};

use crate::node_span::NodeSpan;

struct ListItem<Value: Clone> {
    id: Entity,
    view: ViewRef,
    value: Value,
}

impl<Value: Clone> Clone for ListItem<Value> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            view: ViewRef(self.view.0.clone()),
            value: self.value.clone(),
        }
    }
}

#[doc(hidden)]
#[allow(clippy::needless_range_loop)]
pub struct ForEach<
    Item: Clone,
    ItemIter: Iterator<Item = Item>,
    ItemFn: Fn(&Rcx) -> ItemIter,
    Cmp: Fn(&Item, &Item) -> bool,
    V: IntoView,
    F: Fn(&Item) -> V + Send,
> {
    item_fn: ItemFn,
    items: Vec<ListItem<Item>>,
    cmp: Cmp,
    each: F,
    fallback: Option<ViewRef>,
    fallback_ent: Option<Entity>,
}

#[allow(clippy::needless_range_loop)]
impl<
        Item: Clone,
        ItemIter: Iterator<Item = Item>,
        ItemFn: Fn(&Rcx) -> ItemIter,
        Cmp: Fn(&Item, &Item) -> bool,
        V: IntoView,
        F: Fn(&Item) -> V + Send,
    > ForEach<Item, ItemIter, ItemFn, Cmp, V, F>
{
    pub fn new(item_fn: ItemFn, cmp: Cmp, each: F) -> Self {
        Self {
            item_fn,
            items: Vec::new(),
            each,
            cmp,
            fallback: None,
            fallback_ent: None,
        }
    }

    /// Allow specifying a fallback view to render if there are no items.
    pub fn with_fallback<FB: IntoView>(mut self, fallback: FB) -> Self {
        self.fallback = Some(fallback.into_view());
        self
    }

    /// Uses the sequence of key values to match the previous array items with the updated
    /// array items. Matching items are patched, other items are inserted or deleted.
    ///
    /// # Arguments
    ///
    /// * `bc` - [`BuildContext`] used to build individual elements.
    /// * `prev_state` - Array of view state elements from previous update.
    /// * `prev_range` - The range of elements we are comparing in `prev_state`.
    /// * `next_state` - Array of view state elements to be built.
    /// * `next_range` - The range of elements we are comparing in `next_state`.
    #[allow(clippy::too_many_arguments)]
    fn build_recursive(
        &self,
        world: &mut World,
        view_entity: Entity,
        prev_state: &[ListItem<Item>],
        prev_range: Range<usize>,
        next_items: &[Item],
        next_range: Range<usize>,
        out: &mut Vec<ListItem<Item>>,
    ) -> bool {
        let mut changed = false;

        // Look for longest common subsequence.
        // prev_start and next_start are *relative to the slice*.
        let (prev_start, next_start, lcs_length) = lcs(
            &prev_state[prev_range.clone()],
            &next_items[next_range.clone()],
            |a, b| (self.cmp)(&a.value, b),
        );

        // If there was nothing in common
        if lcs_length == 0 {
            // Raze old elements
            for i in prev_range {
                let prev = &prev_state[i];
                prev.view.raze(prev.id, world);
                changed = true;
            }
            // Build new elements
            for i in next_range {
                changed = true;
                let view = (self.each)(&next_items[i]).into_view();
                out.push(ListItem {
                    id: ViewRef::spawn(&view, view_entity, world),
                    view,
                    value: next_items[i].clone(),
                });
            }
            return changed;
        }

        // Adjust prev_start and next_start to be relative to the entire state array.
        let prev_start = prev_start + prev_range.start;
        let next_start = next_start + next_range.start;

        // Stuff that precedes the LCS.
        if prev_start > prev_range.start {
            if next_start > next_range.start {
                // Both prev and next have entries before lcs, so recurse
                changed |= self.build_recursive(
                    world,
                    view_entity,
                    prev_state,
                    prev_range.start..prev_start,
                    next_items,
                    next_range.start..next_start,
                    out,
                )
            } else {
                // Deletions
                for i in prev_range.start..prev_start {
                    let prev = &prev_state[i];
                    prev.view.raze(prev.id, world);
                    changed = true;
                }
            }
        } else if next_start > next_range.start {
            // Insertions
            for i in next_range.start..next_start {
                let view = (self.each)(&next_items[i]).into_view();
                out.push(ListItem {
                    id: ViewRef::spawn(&view, view_entity, world),
                    view,
                    value: next_items[i].clone(),
                });
                changed = true;
            }
        }

        // For items that match, copy over the view and value.
        for i in 0..lcs_length {
            let prev = &prev_state[prev_start + i];
            out.push(prev.clone());
        }

        // Stuff that follows the LCS.
        let prev_end = prev_start + lcs_length;
        let next_end = next_start + lcs_length;
        if prev_end < prev_range.end {
            if next_end < next_range.end {
                // Both prev and next have entries after lcs, so recurse
                changed |= self.build_recursive(
                    world,
                    view_entity,
                    prev_state,
                    prev_end..prev_range.end,
                    next_items,
                    next_end..next_range.end,
                    out,
                );
            } else {
                // Deletions
                for i in prev_end..prev_range.end {
                    let prev = &prev_state[i];
                    prev.view.raze(prev.id, world);
                    changed = true;
                }
            }
        } else if next_end < next_range.end {
            // Insertions
            for i in next_end..next_range.end {
                let view = (self.each)(&next_items[i]).into_view();
                out.push(ListItem {
                    id: ViewRef::spawn(&view, view_entity, world),
                    view,
                    value: next_items[i].clone(),
                });
                changed = true;
            }
        }

        changed
    }
}

#[allow(clippy::needless_range_loop)]
impl<
        Item: Clone,
        ItemIter: Iterator<Item = Item>,
        ItemFn: Fn(&Rcx) -> ItemIter,
        Cmp: Fn(&Item, &Item) -> bool,
        V: IntoView,
        F: Fn(&Item) -> V + Send,
    > View for ForEach<Item, ItemIter, ItemFn, Cmp, V, F>
{
    fn nodes(&self) -> NodeSpan {
        let child_spans: Vec<NodeSpan> = self.items.iter().map(|item| item.view.nodes()).collect();
        NodeSpan::Fragment(child_spans.into_boxed_slice())
    }

    fn build(&mut self, view_entity: bevy::prelude::Entity, world: &mut World) {
        let mut tracking = TrackingScope::new(world.change_tick());
        self.react(view_entity, world, &mut tracking);
        world
            .entity_mut(view_entity)
            .insert((tracking, Name::new("ForEach")));
        assert!(
            world.entity_mut(view_entity).get::<Parent>().is_some(),
            "ForKeyed should have a parent view"
        );
    }

    fn react(&mut self, view_entity: Entity, world: &mut World, tracking: &mut TrackingScope) {
        let iter = (self.item_fn)(&Rcx::new(world, view_entity, tracking));
        let hint = iter.size_hint().0;
        let items: Vec<Item> = iter.collect();
        let mut next_state: Vec<ListItem<Item>> = Vec::with_capacity(hint);
        let next_len = items.len();
        let prev_len = self.items.len();

        let mut changed = self.build_recursive(
            world,
            view_entity,
            &self.items,
            0..prev_len,
            &items,
            0..next_len,
            &mut next_state,
        );

        // Handle fallback
        if let Some(ref mut fallback) = self.fallback {
            match self.fallback_ent {
                // If there are > 0 items, destroy fallback if present.
                Some(fb_ent) if next_len > 0 => {
                    fallback.raze(fb_ent, world);
                    self.fallback_ent = None;
                    changed = true;
                }

                // If there are no items, render fallback unless already rendered.
                None if next_len == 0 => {
                    self.fallback_ent = Some(ViewRef::spawn(fallback, view_entity, world));
                    changed = true;
                }

                // Otherwise, no change.
                _ => {}
            }
        }

        if changed {
            world.entity_mut(view_entity).insert(DisplayNodeChanged);
        }

        self.items = std::mem::take(&mut next_state);
    }

    fn raze(&mut self, view_entity: Entity, world: &mut World) {
        for entry in self.items.drain(..) {
            entry.view.raze(entry.id, world);
        }
        world.despawn_owned_recursive(view_entity);
    }
}

impl<
        Item: Send + Sync + 'static + Clone,
        ItemIter: 'static + Iterator<Item = Item>,
        ItemFn: Send + Sync + 'static + Fn(&Rcx) -> ItemIter,
        Cmp: Send + Sync + 'static + Fn(&Item, &Item) -> bool,
        V: 'static + IntoView,
        F: Send + Sync + 'static + Fn(&Item) -> V + Send,
    > IntoView for ForEach<Item, ItemIter, ItemFn, Cmp, V, F>
{
    fn into_view(self) -> ViewRef {
        ViewRef::new(self)
    }
}

// #[cfg(test)]
// mod tests {
//     use bevy::ecs::world::World;

//     use super::*;

//     #[test]
//     fn test_update() {
//         let mut world = World::new();
//         let entity = world.spawn_empty().id();
//         let mut bc = BuildContext {
//             world: &mut world,
//             entity,
//         };

//         // Initial render
//         let view = ForKeyed::new(&[1, 2, 3], |item| *item, |item| format!("{}", item));
//         let mut state = view.build(&mut bc);
//         assert_eq!(state.len(), 3);
//         assert_eq!(state[0].key, 1);
//         assert!(state[0].state.is_some());
//         assert_eq!(state[1].key, 2);
//         assert!(state[1].state.is_some());
//         assert_eq!(state[2].key, 3);
//         assert!(state[2].state.is_some());
//         let e1 = state[0].state;

//         // Insert at start
//         let view = ForKeyed::new(&[0, 1, 2, 3], |item| *item, |item| format!("{}", item));
//         view.update(&mut bc, &mut state);
//         assert_eq!(state.len(), 4);
//         assert_eq!(state[0].key, 0);
//         assert_eq!(state[3].key, 3);
//         assert_eq!(state[1].state, e1, "Should be same entity");

//         // Delete at start
//         let view = ForKeyed::new(&[1, 2, 3], |item| *item, |item| format!("{}", item));
//         view.update(&mut bc, &mut state);
//         assert_eq!(state.len(), 3);
//         assert_eq!(state[0].key, 1);
//         assert_eq!(state[2].key, 3);
//         assert_eq!(state[0].state, e1, "Should be same entity");

//         // Insert at end
//         let view = ForKeyed::new(&[1, 2, 3, 4], |item| *item, |item| format!("{}", item));
//         view.update(&mut bc, &mut state);
//         assert_eq!(state.len(), 4);
//         assert_eq!(state[0].key, 1);
//         assert_eq!(state[3].key, 4);
//         assert_eq!(state[0].state, e1, "Should be same entity");

//         // Delete at end
//         let view = ForKeyed::new(&[1, 2, 3], |item| *item, |item| format!("{}", item));
//         view.update(&mut bc, &mut state);
//         assert_eq!(state.len(), 3);
//         assert_eq!(state[0].key, 1);
//         assert_eq!(state[2].key, 3);
//         assert_eq!(state[0].state, e1, "Should be same entity");

//         // Delete in middle
//         let view = ForKeyed::new(&[1, 3], |item| *item, |item| format!("{}", item));
//         view.update(&mut bc, &mut state);
//         assert_eq!(state.len(), 2);
//         assert_eq!(state[0].key, 1);
//         assert_eq!(state[1].key, 3);
//         assert_eq!(state[0].state, e1, "Should be same entity");

//         // Insert in middle
//         let view = ForKeyed::new(&[1, 2, 3], |item| *item, |item| format!("{}", item));
//         view.update(&mut bc, &mut state);
//         assert_eq!(state.len(), 3);
//         assert_eq!(state[0].key, 1);
//         assert_eq!(state[1].key, 2);
//         assert_eq!(state[2].key, 3);
//         assert_eq!(state[0].state, e1, "Should be same entity");

//         // Replace in the middle
//         let view = ForKeyed::new(&[1, 5, 3], |item| *item, |item| format!("{}", item));
//         view.update(&mut bc, &mut state);
//         assert_eq!(state.len(), 3);
//         assert_eq!(state[0].key, 1);
//         assert_eq!(state[1].key, 5);
//         assert_eq!(state[2].key, 3);
//         assert_eq!(state[0].state, e1, "Should be same entity");
//     }
// }
