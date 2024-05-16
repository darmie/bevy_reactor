use std::{
    cell::RefCell,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use bevy::prelude::*;

use crate::{
    callback::{Callback, CallbackFnCell, CallbackFnMutCell},
    derived::{Derived, DerivedCell, ReadDerived, ReadDerivedInternal},
    mutable::{MutableCell, ReadMutable, UpdateMutableCell, WriteMutable},
    tracking_scope::TrackingScope,
    Mutable, Reaction, ReactionCell, Signal,
};

/// An immutable reactive context, used for reactive closures such as derived signals.
pub trait RunContextRead {
    /// Return a reference to the resource of the given type. Calling this function
    /// adds the resource as a dependency of the current tracking scope.
    fn use_resource<T: Resource>(&self) -> &T;

    /// Return a reference to the Component `C` on the given entity. Calling this function
    /// adds the component as a dependency of the current tracking scope.
    fn use_component<C: Component>(&self, entity: Entity) -> Option<&C>;
}

/// A mutable reactive context. This allows write access to reactive data sources.
pub trait RunContextWrite: RunContextRead {
    /// The current Bevy [`World`].
    fn world_mut(&mut self) -> &mut World;

    /// Invoke a callback with the given props.
    ///
    /// Arguments:
    /// * `callback` - The callback to invoke.
    /// * `props` - The props to pass to the callback.
    fn run_callback<P: 'static>(&mut self, callback: Callback<P>, props: P) {
        let world = self.world_mut();
        let tick = world.change_tick();
        let mut tracking = TrackingScope::new(tick);
        let mut cx = Cx::new(world, callback.id, &mut tracking);
        let mut callback_entity = cx.world.entity_mut(callback.id);
        if let Some(mut cell) = callback_entity.get_mut::<CallbackFnCell<P>>() {
            let mut callback_fn = cell.inner.take();
            let callback_box = callback_fn.as_ref().expect("Callback is not present");
            callback_box.call(&mut cx, props);
            let mut callback_entity = cx.world.entity_mut(callback.id);
            callback_entity
                .get_mut::<CallbackFnCell<P>>()
                .unwrap()
                .inner = callback_fn.take();
        } else if let Some(mut cell) = callback_entity.get_mut::<CallbackFnMutCell<P>>() {
            let mut callback_fn = cell.inner.take();
            let callback_box = callback_fn.as_mut().expect("Callback is not present");
            callback_box.call(&mut cx, props);
            let mut callback_entity = cx.world.entity_mut(callback.id);
            callback_entity
                .get_mut::<CallbackFnMutCell<P>>()
                .unwrap()
                .inner = callback_fn.take();
        } else {
            warn!("No callback found for {:?}", callback.id);
        }
    }
}

/// A "setup context" is similar to a reactive context, but can also be used to create
/// reactive data sources such as mutables and effects.
pub trait RunContextSetup<'p> {
    /// The current Bevy [`World`].
    fn world_mut(&mut self) -> &mut World;

    /// Add an owned entity to this tracking scope.
    fn add_owned(&self, mutable: Entity);

    /// Entity that owns this context.
    fn owner(&self) -> Entity;

    /// Set the debug name of the owner entity.
    fn set_owner_name(&mut self, name: &str) {
        let owner = self.owner();
        self.world_mut()
            .entity_mut(owner)
            .insert(Name::new(name.to_string()));
    }

    /// Create a new [`Mutable`] in this context.
    fn create_mutable<T>(&mut self, init: T) -> Mutable<T>
    where
        T: Send + Sync + 'static,
    {
        let owner = self.owner();
        let cell = self
            .world_mut()
            .spawn(MutableCell::<T>(init))
            .set_parent(owner)
            .id();
        let component = self.world_mut().init_component::<MutableCell<T>>();
        self.add_owned(cell);
        Mutable {
            cell,
            component,
            marker: PhantomData,
        }
    }

    /// Create a new [`Callback`] in this context. This holds a `Fn` within an entity.
    ///
    /// Arguments:
    /// * `callback` - The callback function to invoke. This will be called with a single
    ///    parameter, which is a [`Cx`] object. The context may or may not have props.
    fn create_callback<P: 'static, F: Send + Sync + 'static + Fn(&mut Cx, P)>(
        &mut self,
        callback: F,
    ) -> Callback<P> {
        let owner = self.owner();
        let callback = self
            .world_mut()
            .spawn(CallbackFnCell::<P> {
                inner: Some(Box::new(callback)),
            })
            .set_parent(owner)
            .id();
        self.add_owned(callback);
        Callback {
            id: callback,
            marker: PhantomData,
        }
    }

    /// Create a new [`CallbackFnMut`] in this context. This holds a `FnMut` within an entity.
    ///
    /// Arguments:
    /// * `callback` - The callback function to invoke. This will be called with a single
    ///    parameter, which is a [`Cx`] object. The context may or may not have props.
    fn create_callback_mut<P: 'static, F: FnMut(&mut Cx, P)>(&mut self, callback: F) -> Callback<P>
    where
        F: Send + Sync + 'static,
    {
        let owner = self.owner();
        let callback = self
            .world_mut()
            .spawn(CallbackFnMutCell {
                inner: Some(Box::new(callback)),
            })
            .set_parent(owner)
            .id();
        self.add_owned(callback);
        Callback {
            id: callback,
            marker: PhantomData,
        }
    }

    /// Create a new [`Derived`] in this context. This represents a readable signal which
    /// is computed from other signals. The result is not memoized, but is recomputed whenever
    /// the dependencies change.
    ///
    /// Arguments:
    /// * `compute` - The function that computes the output. This will be called with a single
    ///    parameter, which is an [`Rcx`] object.
    fn create_derived<R: 'static, F: Send + Sync + 'static + Fn(&mut Rcx) -> R>(
        &mut self,
        compute: F,
    ) -> Signal<R> {
        let owner = self.owner();
        let derived = self
            .world_mut()
            .spawn(DerivedCell::<R>(Arc::new(compute)))
            .set_parent(owner)
            .id();
        self.add_owned(derived);
        Signal::Derived(Derived {
            id: derived,
            marker: PhantomData,
        })
    }

    /// Create a new [`Memo`] in this context. This represents a readable signal which
    /// is computed from other signals. The result is memoized, which means that downstream
    /// dependants will not be notified unless the output changes.
    ///
    /// Arguments:
    /// * `compute` - The function that computes the output. This will be called with a single
    ///    parameter, which is a [`Cx`] object.
    fn create_memo<
        R: 'static + PartialEq + Send + Sync + Clone,
        F: Send + Sync + 'static + Fn(&mut Cx) -> R,
    >(
        &mut self,
        compute: F,
    ) -> Signal<R> {
        let owner = self.owner();
        let ticks = self.world_mut().change_tick();
        let mut scope = TrackingScope::new(ticks);
        let init = compute(&mut Cx::new(self.world_mut(), owner, &mut scope));
        let mutable = self.create_mutable(init);
        let signal = mutable.signal();
        let reaction = Arc::new(Mutex::new(move |cx: &mut Cx| {
            let prev_value = mutable.get_clone(cx);
            let value = compute(cx);
            if value != prev_value {
                mutable.set_clone(cx, value);
            }
        }));
        self.world_mut().entity_mut(mutable.cell).insert((
            ReactionCell(reaction),
            scope,
            Name::new(format!("Memo::<{}>", std::any::type_name::<R>())),
        ));

        signal
    }

    /// Create an effect. This is a function that is associated with an entity, and which
    /// re-runs whenever any of it's dependencies change.
    ///
    /// Arguments:
    /// * `effect` - The function that computes the output. This will be called with a single
    ///    parameter, which is a [`Cx`] object.
    fn create_effect<F: Send + Sync + 'static + FnMut(&mut Cx)>(&mut self, effect: F) {
        let owner = self.owner();
        let ticks = self.world_mut().change_tick();
        let action = Arc::new(Mutex::new(effect));
        let mut scope = TrackingScope::new(ticks);
        let entity = self.world_mut().spawn_empty().set_parent(owner).id();
        self.add_owned(entity);
        action.lock().unwrap()(&mut Cx::new(self.world_mut(), entity, &mut scope));
        self.world_mut()
            .entity_mut(entity)
            .insert((scope, ReactionCell(action)));
    }
}

impl<F: Send + Sync + 'static + FnMut(&mut Cx)> Reaction for F {
    fn react(&mut self, owner: Entity, world: &mut World, tracking: &mut TrackingScope) {
        let mut cx = Cx::new(world, owner, tracking);
        (self)(&mut cx);
    }
}

/// Cx is a context parameter that is passed to presenters and callbacks. It contains the
/// presenter's properties (passed from the parent presenter), plus a reactive scope and
/// access to reactive data sources in the world.
pub struct Cx<'p, 'w> {
    /// Bevy World
    world: &'w mut World,

    /// The entity that owns the tracking scope (or will own it).
    owner: Entity,

    /// Set of reactive resources referenced by the presenter.
    pub(crate) tracking: RefCell<&'p mut TrackingScope>,
}

impl<'p, 'w> Cx<'p, 'w> {
    pub(crate) fn new(
        world: &'w mut World,
        owner: Entity,
        tracking: &'p mut TrackingScope,
    ) -> Self {
        Self {
            world,
            owner,
            tracking: RefCell::new(tracking),
        }
    }

    /// Access to world from reactive context.
    pub fn world(&self) -> &World {
        self.world
    }

    /// Access to mutable world from reactive context.
    pub fn world_mut(&mut self) -> &mut World {
        self.world
    }

    /// Spawn an empty [`Entity`]. The caller is responsible for despawning the entity.
    pub fn create_entity(&mut self) -> Entity {
        self.world_mut().spawn_empty().id()
    }

    /// Spawn an empty [`Entity`]. The entity will be despawned when the tracking scope is dropped.
    pub fn create_owned_entity(&mut self) -> Entity {
        let entity = self.world_mut().spawn_empty().id();
        self.tracking.borrow_mut().add_owned(entity);
        entity
    }

    /// Return a reference to the Component `C` on the given entity.
    pub fn use_component<C: Component>(&self, entity: Entity) -> Option<&C> {
        let component = self
            .world
            .components()
            .component_id::<C>()
            .expect("Unknown component type");

        match self.world.get_entity(entity) {
            Some(c) => {
                self.tracking
                    .borrow_mut()
                    .track_component_id(entity, component);
                c.get::<C>()
            }
            None => None,
        }
    }

    /// Return a reference to the Component `C` on the owner entity of the current
    /// context, or one of it's ancestors. This searches up the entity tree until it finds
    /// a component of the given type.
    pub fn use_inherited_component<C: Component>(&self) -> Option<&C> {
        let mut entity = self.owner;
        loop {
            let ec = self.use_component(entity);
            if ec.is_some() {
                return ec;
            }
            match self.world.entity(entity).get::<Parent>() {
                Some(parent) => entity = **parent,
                _ => return None,
            }
        }
    }

    // /// Return a reference to the Component `C` on the given entity. This version does not
    // /// add the component to the tracking scope, and is intended for components that update
    // /// frequently.
    // pub fn use_component_untracked<C: Component>(&self, entity: Entity) -> Option<&C> {
    //     match self.bc.world.get_entity(entity) {
    //         Some(c) => c.get::<C>(),
    //         None => None,
    //     }
    // }

    /// Insert a component on the owner entity of the current context. This component can
    /// be accessed by this context any any child contexts via [`use_inherited_component`].
    pub fn insert(&mut self, component: impl Component) {
        let owner = self.owner;
        self.world_mut().entity_mut(owner).insert(component);
    }

    /// Add a cleanup function which is run once before the next reaction, or when the owner
    /// entity for this context is despawned.
    pub fn on_cleanup(&mut self, cleanup: impl FnOnce(&mut World) + Send + Sync + 'static) {
        self.tracking.borrow_mut().add_cleanup(cleanup);
    }
}

impl<'p, 'w> ReadMutable for Cx<'p, 'w> {
    fn read_mutable<T>(&self, mutable: &Mutable<T>) -> T
    where
        T: Send + Sync + Copy + 'static,
    {
        self.tracking
            .borrow_mut()
            .track_component_id(mutable.cell, mutable.component);
        self.world.read_mutable(mutable)
    }

    fn read_mutable_clone<T>(&self, mutable: &Mutable<T>) -> T
    where
        T: Send + Sync + Clone + 'static,
    {
        self.tracking
            .borrow_mut()
            .track_component_id(mutable.cell, mutable.component);
        self.world.read_mutable_clone(mutable)
    }

    fn read_mutable_as_ref<T>(&self, mutable: &Mutable<T>) -> &T
    where
        T: Send + Sync + 'static,
    {
        self.tracking
            .borrow_mut()
            .track_component_id(mutable.cell, mutable.component);
        self.world.read_mutable_as_ref(mutable)
    }

    fn read_mutable_map<T, U, F: Fn(&T) -> U>(&self, mutable: &Mutable<T>, f: F) -> U
    where
        T: Send + Sync + 'static,
    {
        self.tracking
            .borrow_mut()
            .track_component_id(mutable.cell, mutable.component);
        self.world.read_mutable_map(mutable, f)
    }
}

impl<'p, 'w> WriteMutable for Cx<'p, 'w> {
    fn write_mutable<T>(&mut self, mutable: Entity, value: T)
    where
        T: Send + Sync + Copy + PartialEq + 'static,
    {
        self.world.write_mutable(mutable, value);
    }

    fn write_mutable_clone<T>(&mut self, mutable: Entity, value: T)
    where
        T: Send + Sync + Clone + PartialEq + 'static,
    {
        self.world.write_mutable_clone(mutable, value);
    }
}

impl<'p, 'w> ReadDerived for Cx<'p, 'w> {
    fn read_derived<R>(&self, derived: &Derived<R>) -> R
    where
        R: Send + Sync + Copy + 'static,
    {
        self.world
            .read_derived_with_scope(derived.id, &mut self.tracking.borrow_mut())
    }

    fn read_derived_clone<R>(&self, derived: &Derived<R>) -> R
    where
        R: Send + Sync + Clone + 'static,
    {
        self.world
            .read_derived_clone_with_scope(derived.id, &mut self.tracking.borrow_mut())
    }

    fn read_derived_map<R, U, F: Fn(&R) -> U>(&self, derived: &Derived<R>, f: F) -> U
    where
        R: Send + Sync + 'static,
    {
        self.world
            .read_derived_map_with_scope(derived.id, &mut self.tracking.borrow_mut(), f)
    }
}

impl<'p, 'w> RunContextRead for Cx<'p, 'w> {
    fn use_resource<T: Resource>(&self) -> &T {
        self.tracking.borrow_mut().track_resource::<T>(self.world);
        self.world.resource::<T>()
    }

    fn use_component<C: Component>(&self, entity: Entity) -> Option<&C> {
        self.tracking
            .borrow_mut()
            .track_component::<C>(entity, self.world);
        self.world.entity(entity).get::<C>()
    }
}

impl<'p, 'w> RunContextWrite for Cx<'p, 'w> {
    fn world_mut(&mut self) -> &mut World {
        self.world
    }
}

impl<'p, 'w> RunContextSetup<'p> for Cx<'p, 'w> {
    fn world_mut(&mut self) -> &mut World {
        self.world
    }

    fn owner(&self) -> Entity {
        self.owner
    }

    fn add_owned(&self, entity: Entity) {
        self.tracking.borrow_mut().add_owned(entity);
    }
}

/// Immutable reactive context, used for reactive closures such as derived signals.
/// This is a stripped down version of [`Cx`] that does not allow creating new reactions,
/// and which has no parameters.
pub struct Rcx<'p, 'w> {
    /// Bevy World
    pub(crate) world: &'w World,

    /// The entity that owns the tracking scope (or will own it).
    pub(crate) owner: Entity,

    /// Set of reactive resources referenced by the presenter.
    pub(crate) tracking: RefCell<&'p mut TrackingScope>,
}

impl<'p, 'w> Rcx<'p, 'w> {
    /// Create a new read-only reactive context.
    pub fn new(world: &'w World, owner: Entity, tracking: &'p mut TrackingScope) -> Self {
        Self {
            world,
            owner,
            tracking: RefCell::new(tracking),
        }
    }

    /// Access to immutable world from reactive context.
    pub fn world(&self) -> &World {
        self.world
    }

    /// Return a reference to the Component `C` on the owner entity of the current
    /// context, or one of it's ancestors. This searches up the entity tree until it finds
    /// a component of the given type.
    pub fn use_inherited_component<C: Component>(&self) -> Option<&C> {
        let mut entity = self.owner;
        loop {
            let ec = self.use_component(entity);
            if ec.is_some() {
                return ec;
            }
            match self.world.entity(entity).get::<Parent>() {
                Some(parent) => entity = **parent,
                _ => return None,
            }
        }
    }

    /// Add a cleanup function which is run once before the next reaction, or when the owner
    /// entity for this context is despawned.
    pub fn on_cleanup(&mut self, cleanup: impl FnOnce(&mut World) + Send + Sync + 'static) {
        self.tracking.borrow_mut().add_cleanup(cleanup);
    }
}

impl<'p, 'w> ReadMutable for Rcx<'p, 'w> {
    fn read_mutable<T>(&self, mutable: &Mutable<T>) -> T
    where
        T: Send + Sync + Copy + 'static,
    {
        self.tracking
            .borrow_mut()
            .track_component_id(mutable.cell, mutable.component);
        self.world.read_mutable(mutable)
    }

    fn read_mutable_clone<T>(&self, mutable: &Mutable<T>) -> T
    where
        T: Send + Sync + Clone + 'static,
    {
        self.tracking
            .borrow_mut()
            .track_component_id(mutable.cell, mutable.component);
        self.world.read_mutable_clone(mutable)
    }

    fn read_mutable_as_ref<T>(&self, mutable: &Mutable<T>) -> &T
    where
        T: Send + Sync + 'static,
    {
        self.tracking
            .borrow_mut()
            .track_component_id(mutable.cell, mutable.component);
        self.world.read_mutable_as_ref(mutable)
    }

    fn read_mutable_map<T, U, F: Fn(&T) -> U>(&self, mutable: &Mutable<T>, f: F) -> U
    where
        T: Send + Sync + 'static,
    {
        self.tracking
            .borrow_mut()
            .track_component_id(mutable.cell, mutable.component);
        self.world.read_mutable_map(mutable, f)
    }
}

impl<'p, 'w> ReadDerived for Rcx<'p, 'w> {
    fn read_derived<R>(&self, derived: &Derived<R>) -> R
    where
        R: Send + Sync + Copy + 'static,
    {
        self.world
            .read_derived_with_scope(derived.id, &mut self.tracking.borrow_mut())
    }

    fn read_derived_clone<R>(&self, derived: &Derived<R>) -> R
    where
        R: Send + Sync + Clone + 'static,
    {
        self.world
            .read_derived_clone_with_scope(derived.id, &mut self.tracking.borrow_mut())
    }

    fn read_derived_map<R, U, F: Fn(&R) -> U>(&self, derived: &Derived<R>, f: F) -> U
    where
        R: Send + Sync + 'static,
    {
        self.world
            .read_derived_map_with_scope(derived.id, &mut self.tracking.borrow_mut(), f)
    }
}

impl<'p, 'w> RunContextRead for Rcx<'p, 'w> {
    fn use_resource<T: Resource>(&self) -> &T {
        self.tracking.borrow_mut().track_resource::<T>(self.world);
        self.world.resource::<T>()
    }

    fn use_component<C: Component>(&self, entity: Entity) -> Option<&C> {
        self.tracking
            .borrow_mut()
            .track_component::<C>(entity, self.world);
        self.world.entity(entity).get::<C>()
    }
}

impl ReadMutable for World {
    fn read_mutable<T>(&self, mutable: &Mutable<T>) -> T
    where
        T: Send + Sync + Copy + 'static,
    {
        let mutable_entity = self.entity(mutable.cell);
        mutable_entity.get::<MutableCell<T>>().unwrap().0
    }

    fn read_mutable_clone<T>(&self, mutable: &Mutable<T>) -> T
    where
        T: Send + Sync + Clone + 'static,
    {
        let mutable_entity = self.entity(mutable.cell);
        mutable_entity.get::<MutableCell<T>>().unwrap().0.clone()
    }

    fn read_mutable_as_ref<T>(&self, mutable: &Mutable<T>) -> &T
    where
        T: Send + Sync + 'static,
    {
        let mutable_entity = self.entity(mutable.cell);
        &mutable_entity.get::<MutableCell<T>>().unwrap().0
    }

    fn read_mutable_map<T, U, F: Fn(&T) -> U>(&self, mutable: &Mutable<T>, f: F) -> U
    where
        T: Send + Sync + 'static,
    {
        let mutable_entity = self.entity(mutable.cell);
        f(&mutable_entity.get::<MutableCell<T>>().unwrap().0)
    }
}

impl WriteMutable for World {
    /// Write the value of a mutable variable using Copy semantics. Does nothing if
    /// the value being set matches the existing value.
    fn write_mutable<T>(&mut self, mutable: Entity, value: T)
    where
        T: Send + Sync + PartialEq + 'static,
    {
        self.commands().add(UpdateMutableCell { mutable, value });
    }

    /// Write the value of a mutable variable using Clone semantics. Does nothing if the
    /// value being set matches the existing value.
    fn write_mutable_clone<T>(&mut self, mutable: Entity, value: T)
    where
        T: Send + Sync + Clone + PartialEq + 'static,
    {
        self.commands().add(UpdateMutableCell { mutable, value });
    }
}

impl ReadDerived for World {
    fn read_derived<R>(&self, derived: &Derived<R>) -> R
    where
        R: Send + Sync + Copy + 'static,
    {
        let ticks = self.read_change_tick();
        let mut scope = TrackingScope::new(ticks);
        self.read_derived_with_scope(derived.id, &mut scope)
    }

    fn read_derived_clone<R>(&self, derived: &Derived<R>) -> R
    where
        R: Send + Sync + Clone + 'static,
    {
        let ticks = self.read_change_tick();
        let mut scope = TrackingScope::new(ticks);
        self.read_derived_clone_with_scope(derived.id, &mut scope)
    }

    fn read_derived_map<R, U, F: Fn(&R) -> U>(&self, derived: &Derived<R>, f: F) -> U
    where
        R: Send + Sync + 'static,
    {
        let ticks = self.read_change_tick();
        let mut scope = TrackingScope::new(ticks);
        self.read_derived_map_with_scope(derived.id, &mut scope, f)
    }
}

impl ReadDerivedInternal for World {
    fn read_derived_with_scope<R>(&self, derived: Entity, scope: &mut TrackingScope) -> R
    where
        R: Send + Sync + Copy + 'static,
    {
        let derived_entity = self.entity(derived);
        match derived_entity.get::<DerivedCell<R>>() {
            Some(cell) => {
                let derived_fn = cell.0.clone();
                let mut cx = Rcx::new(self, derived, scope);
                derived_fn.call(&mut cx)
            }
            _ => panic!("No derived found for {:?}", derived),
        }
    }

    fn read_derived_clone_with_scope<R>(&self, derived: Entity, scope: &mut TrackingScope) -> R
    where
        R: Send + Sync + Clone + 'static,
    {
        let derived_entity = self.entity(derived);
        match derived_entity.get::<DerivedCell<R>>() {
            Some(cell) => {
                let derived_fn = cell.0.clone();
                let mut cx = Rcx::new(self, derived, scope);
                derived_fn.call(&mut cx).clone()
            }
            _ => panic!("No derived found for {:?}", derived),
        }
    }

    fn read_derived_map_with_scope<R, U, F: Fn(&R) -> U>(
        &self,
        derived: Entity,
        scope: &mut TrackingScope,
        f: F,
    ) -> U
    where
        R: Send + Sync + 'static,
    {
        let derived_entity = self.entity(derived);
        match derived_entity.get::<DerivedCell<R>>() {
            Some(cell) => {
                let derived_fn = cell.0.clone();
                let mut cx = Rcx::new(self, derived, scope);
                f(&derived_fn.call(&mut cx))
            }
            _ => panic!("No derived found for {:?}", derived),
        }
    }
}

impl RunContextRead for World {
    fn use_resource<T: Resource>(&self) -> &T {
        self.resource::<T>()
    }

    fn use_component<C: Component>(&self, entity: Entity) -> Option<&C> {
        self.entity(entity).get::<C>()
    }
}

impl RunContextWrite for World {
    fn world_mut(&mut self) -> &mut World {
        self
    }
}
