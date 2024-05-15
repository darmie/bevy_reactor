use std::sync::{Arc, Mutex};

use bevy::ecs::component::Component;

use crate::reaction::Reaction;

/// Component used to hold a reference to a [`Reaction`].
#[derive(Component, Clone)]
pub struct ReactionHandle<T: Reaction+ Sync + Send + 'static>(pub(crate) Arc<Mutex<T>>);