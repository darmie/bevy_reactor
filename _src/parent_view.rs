use crate::{IntoView, ViewRef};
use bevy::ecs::{entity::Entity, world::World};
use impl_trait_for_tuples::*;
use smallvec::SmallVec;

/// A reference to an array of child views.
#[derive(Default, Clone)]
pub struct ChildArray(pub(crate) SmallVec<[ViewRef; 4]>);

/// A view which can contain child views. This will generate child entities when spawned.
pub trait ParentView: Sized {
    /// Get the child views for this element.
    fn get_children(&self) -> &Vec<ChildView>;

    /// Get a mutable reference to the child views for this element.
    fn get_children_mut(&mut self) -> &mut Vec<ChildView>;

    /// Return a flat list of child entities derived from the child views.
    fn child_entities(&self) -> Vec<Entity> {
        let mut count: usize = 0;
        for child in self.get_children().iter() {
            count += child.view.nodes().count();
        }

        let mut flat: Vec<Entity> = Vec::with_capacity(count);
        for child in self.get_children().iter() {
            child.view.nodes().flatten(&mut flat);
        }

        flat
    }

    /// Set the child views for this element.
    fn children<V: ChildViewTuple>(mut self, views: V) -> Self {
        if !self.get_children().is_empty() {
            panic!("Children already set");
        }
        self.get_children_mut()
            .extend(views.to_child_array().0.iter().map(|v| ChildView {
                view: v.clone(),
                entity: None,
            }));
        self
    }

    /// Set a single child view for this element.
    fn child(mut self, view: &ViewRef) -> Self {
        self.get_children_mut().push(ChildView {
            view: view.clone(),
            entity: None,
        });
        self
    }

    /// Add a child views to this element.
    fn append_child(mut self, view: &ViewRef) -> Self {
        self.get_children_mut().push(ChildView {
            view: view.clone(),
            entity: None,
        });
        self
    }

    /// Raze all child views.
    fn raze_children(&mut self, world: &mut World) {
        // Raze all child views
        for child in self.get_children_mut().drain(..) {
            // Calling `raze` on the child view will despawn the child entity.
            let inner = child.view;
            inner.raze(child.entity.unwrap(), world);
        }
    }
}

/// Used by widgets to track the entities created by their children.
pub struct ChildView {
    /// The view handle for generating the child entity.
    pub view: ViewRef,
    /// The entity id for the child entity.
    pub entity: Option<Entity>,
}

/// A tuple of [`View`]s which can be converted into a [`Vec<ViewRef>`].
#[doc(hidden)]
pub trait ChildViewTuple {
    #[doc(hidden)]
    fn flatten(self, out: &mut ChildArray);

    /// Convert this tuple of views into a flat array.
    fn to_child_array(self) -> ChildArray;
}

impl<I: IntoView> ChildViewTuple for I {
    fn flatten(self, out: &mut ChildArray) {
        out.0.push(self.into_view());
    }

    fn to_child_array(self) -> ChildArray {
        let mut out = ChildArray::default();
        self.flatten(&mut out);
        out
    }
}

impl ChildViewTuple for Vec<ViewRef> {
    fn flatten(self, out: &mut ChildArray) {
        out.0.extend(self);
    }

    fn to_child_array(self) -> ChildArray {
        ChildArray(SmallVec::from_vec(self))
    }
}

impl ChildViewTuple for ChildArray {
    fn flatten(self, out: &mut ChildArray) {
        out.0.extend(self.0);
    }

    fn to_child_array(self) -> ChildArray {
        self
    }
}

#[impl_for_tuples(1, 15)]
#[tuple_types_custom_trait_bound(ChildViewTuple)]
impl ChildViewTuple for Tuple {
    fn flatten(self, out: &mut ChildArray) {
        for_tuples!(#( self.Tuple.flatten(out); )*)
    }

    fn to_child_array(self) -> ChildArray {
        let mut out = ChildArray::default();
        self.flatten(&mut out);
        out
    }
}
