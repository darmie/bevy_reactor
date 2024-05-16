use std::sync::Arc;

use bevy::prelude::*;

use crate::{Rcx, TrackingScope};

pub(crate) trait DerivedFnRef<R> {
    fn call(&self, cx: &mut Rcx) -> R;
}

impl<R, F: Fn(&mut Rcx) -> R> DerivedFnRef<R> for F {
    fn call(&self, cx: &mut Rcx) -> R {
        self(cx)
    }
}

/// Contains a boxed, type-erased function which returns a reactive result.
#[derive(Component)]
pub(crate) struct DerivedCell<R>(pub(crate) Arc<dyn DerivedFnRef<R> + Send + Sync>);

/// A [`Derived`] is a readonly value that is computed from other signals.
#[derive(PartialEq)]
pub struct Derived<R> {
    pub(crate) id: Entity,
    pub(crate) marker: std::marker::PhantomData<R>,
}

impl<T> Copy for Derived<T> {}
impl<T> Clone for Derived<T> {
    fn clone(&self) -> Self {
        *self
    }
}

/// An immutable reactive context, used for reactive closures such as derived signals.
pub trait ReadDerived {
    /// Read the value of a derived signal using Copy semantics. This adds any dependencies of
    /// the derived signal to the current tracking scope.
    fn read_derived<R>(&self, derived: &Derived<R>) -> R
    where
        R: Send + Sync + Copy + 'static;

    /// Read the value of a mutable variable using Copy semantics. This adds any dependencies of
    /// the derived signal to the current tracking scope.
    fn read_derived_clone<R>(&self, derived: &Derived<R>) -> R
    where
        R: Send + Sync + Clone + 'static;

    /// Read the value of a mutable variable using a mapping function. This adds any dependencies of
    /// the derived signal to the current tracking scope.
    fn read_derived_map<R, U, F: Fn(&R) -> U>(&self, derived: &Derived<R>, f: F) -> U
    where
        R: Send + Sync + 'static;
}

/// Trait used to implement reading of derives while passing an explicit tracking scope.
pub(crate) trait ReadDerivedInternal {
    /// Read the value of a derived signal using Copy semantics. This adds any dependencies of
    /// the derived signal to the current tracking scope.
    fn read_derived_with_scope<R>(&self, derived: Entity, scope: &mut TrackingScope) -> R
    where
        R: Send + Sync + Copy + 'static;

    /// Read the value of a mutable variable using Copy semantics. This adds any dependencies of
    /// the derived signal to the current tracking scope.
    fn read_derived_clone_with_scope<R>(&self, derived: Entity, scope: &mut TrackingScope) -> R
    where
        R: Send + Sync + Clone + 'static;

    /// Read the value of a mutable variable using a mapping function. This adds any dependencies of
    /// the derived signal to the current tracking scope.
    fn read_derived_map_with_scope<R, U, F: Fn(&R) -> U>(
        &self,
        derived: Entity,
        scope: &mut TrackingScope,
        f: F,
    ) -> U
    where
        R: Send + Sync + 'static;
}
