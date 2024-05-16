use std::sync::{Arc, Mutex};

use bevy::{
    prelude::*,
    ecs::{
        entity::Entity,
        world::{Command, World},
    },
    hierarchy::BuildWorldChildren,
};

use crate::{element::Element, text::{TextComputed, TextStatic}};

use bevy_reactor_core::{
    Cx, DespawnScopes,  NodeSpan,  Signal, TrackingScope
};

use bevy::ecs::bundle::Bundle;
use bevy_reactor_core::EffectTarget;
use bevy_reactor_style::{ApplyStylesEffect, StyleTuple, WithStyles};

/// Trait that defines a view, which is a template that constructs a hierarchy of
/// entities and components.
///
/// Lifecycle: To create a view, use [`ViewHandle::spawn`]. This creates an entity to hold the view,
/// and which drives the reaction system. When the view is no longer needed, call [`View::raze`].
/// This will destroy the view entity, and all of its children and display nodes.
#[allow(unused_variables)]
pub trait View {
    /// Returns the display nodes produced by this `View`.
    fn nodes(&self) -> NodeSpan;

    /// Initialize the view, creating any entities needed.
    ///
    /// Arguments:
    /// * `view_entity`: The entity that owns this view.
    /// * `world`: The Bevy world.
    fn build(&mut self, view_entity: Entity, world: &mut World);

    /// Update the view, reacting to changes in dependencies. This is optional, and need only
    /// be implemented for views that are reactive.
    fn react(&mut self, view_entity: Entity, world: &mut World, tracking: &mut TrackingScope) {}

    /// Destroy the view, including the display nodes, and all descendant views.
    fn raze(&mut self, view_entity: Entity, world: &mut World);

    /// Notification from child views that the child display nodes have changed and need
    /// to be re-attached to the parent. This is optional, and need only be implemented for
    /// views which have display nodes that have child display nodes (like [`Element`]).
    ///
    /// Returns `true` if the view was able to update its display nodes. If it returns `false`,
    /// then it means that this view is only a thin wrapper for other views, and doesn't actually
    /// have any display nodes of its own, in which case the parent view will need to handle the
    /// change.
    fn children_changed(&mut self, view_entity: Entity, world: &mut World) -> bool {
        false
    }
}

impl<B: Bundle + Default> WithStyles for Element<B> {
    fn style<S: StyleTuple + 'static>(mut self, styles: S) -> Self {
        self.add_effect(Box::new(ApplyStylesEffect { styles }));
        self
    }
}


#[derive(Component)]
/// Component which holds the top level of the view hierarchy.
pub struct ViewRoot(pub Arc<Mutex<dyn View + Sync + Send + 'static>>);

impl ViewRoot {
    /// Construct a new [`ViewRoot`].
    pub fn new(view: impl View + Sync + Send + 'static) -> Self {
        Self(Arc::new(Mutex::new(view)))
    }

    /// Despawn the view, including the display nodes, and all descendant views.
    pub fn despawn(&mut self, root: Entity, world: &mut World) {
        self.0.lock().unwrap().raze(root, world);
        world.entity_mut(root).despawn();
    }
}

/// Command which can be used to despawn a view root and all of it's contents.
pub struct DespawnViewRoot(Entity);

impl DespawnViewRoot {
    /// Construct a new [`DespawnViewRoot`] command.
    pub fn new(entity: Entity) -> Self {
        Self(entity)
    }
}

impl Command for DespawnViewRoot {
    fn apply(self, world: &mut World) {
        let entt = world.entity_mut(self.0);
        let Some(root) = entt.get::<ViewRoot>() else {
            return;
        };
        let handle = root.0.clone();
        let mut view = handle.lock().unwrap();
        view.raze(self.0, world);
        // let entt = world.entity_mut(self.0);
        // entt.despawn();
    }
}


/// Component used to hold a reference to a [`View`].
#[derive(Component, Clone)]
pub struct ViewHandle(pub Arc<Mutex<dyn View+ Sync + Send + 'static>>);

/// A reference to a [`View`] which can be passed around as a parameter.
pub struct ViewRef(pub(crate) Arc<Mutex<dyn View + Sync + Send + 'static>>);


impl ViewRef {
    /// Construct a new [`ViewRef`] from a [`View`].
    pub fn new(view:impl View + Sync + Send + 'static) -> Self  {
        Self(Arc::new(Mutex::new(view)))
    }

    /// Given a view template, construct a new view. This creates an entity to hold the view
    /// and the view handle, and then calls [`View::build`] on the view. The resuling entity
    /// is part of the template invocation hierarchy, it is not a display node.
    pub fn spawn(view: &ViewRef, parent: Entity, world: &mut World) -> Entity {
        let mut child_ent = world.spawn(ViewHandle(view.0.clone()));
        child_ent.set_parent(parent);
        let id = child_ent.id();
        view.0.lock().unwrap().build(child_ent.id(), world);
        id
    }

    /// Returns the display nodes produced by this `View`.
    pub fn nodes(&self) -> NodeSpan {
        self.0.lock().unwrap().nodes()
    }

    /// Destroy the view, including the display nodes, and all descendant views.
    pub fn raze(&self, view_entity: Entity, world: &mut World) {
        self.0.lock().unwrap().raze(view_entity, world);
    }
}

impl Clone for ViewRef {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Default for ViewRef {
    fn default() -> Self {
        Self::new(EmptyView)
    }
}

/// Trait that allows a type to be converted into a [`ViewRef`].
pub trait IntoView {
    /// Convert the type into a [`ViewRef`].
    fn into_view(self) -> ViewRef;
}

impl IntoView for ViewRef {
    fn into_view(self) -> ViewRef {
        self
    }
}

impl IntoView for () {
    fn into_view(self) -> ViewRef {
        ViewRef::new(EmptyView)
    }
}

impl IntoView for &str {
    fn into_view(self) -> ViewRef {
        ViewRef::new(TextStatic::new(self.to_string()))
    }
}

impl IntoView for Signal<String> {
    fn into_view(self) -> ViewRef {
        ViewRef::new(TextComputed::new(move |rcx| self.get_clone(rcx)))
    }
}

impl IntoView for String {
    fn into_view(self) -> ViewRef {
        ViewRef::new(TextStatic::new(self))
    }
}

impl<V: IntoView> IntoView for Option<V> {
    fn into_view(self) -> ViewRef {
        match self {
            Some(v) => v.into_view(),
            None => ViewRef::new(EmptyView),
        }
    }
}

#[derive(Component)]
/// Marker component used to signal that a view's output nodes have changed.
pub struct DisplayNodeChanged;

/// A `[View]` which displays nothing - can be used as a placeholder.
pub struct EmptyView;

#[allow(unused_variables)]
impl View for EmptyView {
    fn nodes(&self) -> NodeSpan {
        NodeSpan::Empty
    }

    fn build(&mut self, view_entity: Entity, world: &mut World) {}
    fn raze(&mut self, view_entity: Entity, world: &mut World) {}
}

/// Trait that defines a factory object that can construct a [`View`] from a reactive context.
/// Similar to a `PresenterFn`, but allows the template to be defined as a type, rather than as
/// a function.
pub trait ViewTemplate {
    /// Create the view for the control.
    fn create(&self, cx: &mut Cx) -> impl IntoView;

    /// Associate this view template with a state object that tracks the nodes created by
    /// the view. Consumes the template.
    fn to_root(self) -> ViewRoot
    where
        Self: Sized + Send + Sync + 'static,
    {
        ViewRoot::new(ViewTemplateState::new(self))
    }
}

impl<VT: ViewTemplate + Send + Sync + 'static> IntoView for VT {
    fn into_view(self) -> ViewRef {
        ViewRef::new(ViewTemplateState::new(self))
    }
}

/// Holds a [`ViewTemplate`], and the entity and output nodes created by the [`View`] produced
/// by the factory.
pub struct ViewTemplateState<VF: ViewTemplate> {
    /// Reference to factory object.
    template: VF,

    /// The entity representing the output (inner) view generated by the template (not the entity
    /// that contains the template state.)
    output_entity: Option<Entity>,

    /// Display nodes generated by the view generated by this factory.
    nodes: NodeSpan,
}

impl<W: ViewTemplate> ViewTemplateState<W> {
    /// Construct a new `WidgetInstance`.
    pub fn new(template: W) -> Self {
        Self {
            template,
            output_entity: None,
            nodes: NodeSpan::Empty,
        }
    }
}

impl<W: ViewTemplate> View for ViewTemplateState<W> {
    fn nodes(&self) -> NodeSpan {
        self.nodes.clone()
    }

    fn build(&mut self, view_entity: Entity, world: &mut World) {
        assert!(self.output_entity.is_none());
        let mut tracking = TrackingScope::new(world.change_tick());
        let mut cx = Cx::new(world, view_entity, &mut tracking);
        let view = self.template.create(&mut cx).into_view();
        let inner = world.spawn_empty().set_parent(view_entity).id();
        view.0.lock().unwrap().build(inner, world);
        self.nodes = view.nodes();
        world.entity_mut(inner).insert(ViewHandle(view.0));
        world.entity_mut(view_entity).insert(tracking);
        self.output_entity = Some(inner);
    }

    fn raze(&mut self, view_entity: Entity, world: &mut World) {
        assert!(self.output_entity.is_some());
        let mut entt = world.entity_mut(self.output_entity.unwrap());
        if let Some(handle) = entt.get_mut::<ViewHandle>() {
            // Despawn the inner view.
            handle.0.clone().lock().unwrap().raze(entt.id(), world);
        };
        self.output_entity = None;
        world.despawn_owned_recursive(view_entity);
    }

    fn children_changed(&mut self, _view_entity: Entity, world: &mut World) -> bool {
        // Update cached nodes
        if let Some(handle) = world
            .entity(self.output_entity.unwrap())
            .get::<ViewHandle>()
        {
            self.nodes = handle.0.lock().unwrap().nodes();
        };
        false
    }
}



