use bevy::{ecs::system::SystemParam, prelude::*};

use crate::Cx;

pub(crate) trait CallbackFnRef<P> {
    fn call(&self, cx: &mut Cx, props: P);
}

impl<P, F: Fn(&mut Cx, P)> CallbackFnRef<P> for F {
    fn call(&self, cx: &mut Cx, props: P) {
        self(cx, props);
    }
}

pub(crate) trait CallbackFnMutRef<P> {
    fn call(&mut self, cx: &mut Cx, props: P);
}

impl<P, F: FnMut(&mut Cx, P)> CallbackFnMutRef<P> for F {
    fn call(&mut self, cx: &mut Cx, props: P) {
        self(cx, props);
    }
}

/// Contains a boxed, type-erased callback.
#[derive(Component)]
pub(crate) struct CallbackFnCell<P> {
    pub(crate) inner: Option<Box<dyn CallbackFnRef<P> + Send + Sync>>,
}

#[derive(Component)]
pub(crate) struct CallbackFnMutCell<P> {
    pub(crate) inner: Option<Box<dyn CallbackFnMutRef<P> + Send + Sync>>,
}

/// Contains a reference to a callback. `P` is the type of the props.
#[derive(PartialEq)]
pub struct Callback<P = ()> {
    pub(crate) id: Entity,
    pub(crate) marker: std::marker::PhantomData<P>,
}

impl<P> Copy for Callback<P> {}
impl<P> Clone for Callback<P> {
    fn clone(&self) -> Self {
        *self
    }
}

/// An event which will call a callback with the given props.
#[derive(Event)]
#[allow(dead_code)] // For now
pub(crate) struct DeferredCall<P> {
    pub(crate) receiver: Entity,
    pub(crate) props: P,
}

/// Type that allows us to call a callback without having a reference to a `World`.
/// These calls are deferred until the next frame.
#[derive(SystemParam)]
pub struct CallDeferred<'w, P: Send + Sync + 'static = ()> {
    pub(crate) writer: EventWriter<'w, DeferredCall<P>>,
}

impl<'w, P: Send + Sync + 'static> CallDeferred<'w, P> {
    /// Call the callback with the given props. This will be processed on the next frame.
    pub fn call(&mut self, callback: Callback<P>, props: P) {
        self.writer.send(DeferredCall {
            receiver: callback.id,
            props,
        });
    }
}

// #[allow(dead_code)] // For now
// /// System that runs callbacks from an event listener.
// pub fn run_deferred_callbacks<P: 'static + Send + Sync>(world: &mut World) {
//     let tick = world.change_tick();
//     let mut events = world.get_resource_mut::<Events<DeferredCall<P>>>().unwrap();
//     let events = events.drain().collect::<Vec<_>>();
//     let mut tracking = TrackingScope::new(tick);
//     for event in events {
//         let mut callback_entity = world.entity_mut(event.receiver);
//         if let Some(mut callback_cmp) = callback_entity.get_mut::<CallbackFnCell<P>>() {
//             let mut callback_fn = callback_cmp.inner.take();
//             let callback_box = callback_fn.as_ref().expect("Callback is not present");
//             let mut cx = Cx::new(event.props, world, &mut tracking);
//             callback_box.call(&mut cx);
//             let mut callback_entity = world.entity_mut(event.receiver);
//             callback_entity
//                 .get_mut::<CallbackFnCell<P>>()
//                 .unwrap()
//                 .inner = callback_fn.take();
//         } else if let Some(mut callback_cmp) = callback_entity.get_mut::<CallbackFnMutCell<P>>() {
//             let mut callback_fn = callback_cmp.inner.take();
//             let callback_box = callback_fn.as_mut().expect("Callback is not present");
//             let mut cx = Cx::new(event.props, world, &mut tracking);
//             callback_box.call(&mut cx);
//             let mut callback_entity = world.entity_mut(event.receiver);
//             callback_entity
//                 .get_mut::<CallbackFnMutCell<P>>()
//                 .unwrap()
//                 .inner = callback_fn.take();
//         } else {
//             warn!("No callback found for {:?}", event.receiver);
//         }
//     }
// }
