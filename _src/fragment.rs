use bevy::prelude::*;

use crate::{
    node_span::NodeSpan, parent_view::ChildViewTuple, view::View, ChildView, DespawnScopes,
    IntoView, ViewRef,
};

/// A `Fragment` represents a group of one or more child views which can be inserted inline
/// within a parent view. A parent view which contains a `Fragment` will render the child views of
/// the `Fragment` in place of the `Fragment` itself. This is useful in cases where a function's
/// return type only allows a single view to be returned, but you want to return multiple views.
#[derive(Default)]
pub struct Fragment {
    /// Children of this fragment.
    children: Vec<ChildView>,
}

impl Fragment {
    /// Construct a new `Fragment`.
    pub fn new<V: ChildViewTuple>(views: V) -> Self {
        let child_views = views.to_child_array().0;
        Self {
            children: child_views
                .iter()
                .map(|v| ChildView {
                    view: v.clone(),
                    entity: None,
                })
                .collect(),
        }
    }

    /// Construct a new `Fragment` from an array of `ViewRef`s.
    pub fn from_slice(views: &[ViewRef]) -> Self {
        Self {
            children: views
                .iter()
                .map(|v| ChildView {
                    view: v.clone(),
                    entity: None,
                })
                .collect(),
        }
    }
}

impl View for Fragment {
    fn nodes(&self) -> NodeSpan {
        let child_spans: Vec<NodeSpan> =
            self.children.iter().map(|item| item.view.nodes()).collect();
        NodeSpan::Fragment(child_spans.into_boxed_slice())
    }

    fn build(&mut self, view_entity: Entity, world: &mut World) {
        // Build child nodes.
        for child in self.children.iter_mut() {
            child.entity = Some(ViewRef::spawn(&child.view, view_entity, world));
        }
    }

    fn raze(&mut self, view_entity: Entity, world: &mut World) {
        // Raze all child views
        for child in self.children.iter_mut() {
            child.view.raze(child.entity.unwrap(), world);
            child.entity = None;
            // Child raze() will despawn itself.
        }

        world.despawn_owned_recursive(view_entity);
    }
}

impl IntoView for Fragment {
    fn into_view(self) -> ViewRef {
        ViewRef::new(self)
    }
}
