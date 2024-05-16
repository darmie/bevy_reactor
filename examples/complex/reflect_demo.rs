use std::sync::Arc;

use bevy::prelude::*;
use bevy_reactor_core::*;
use bevy_reactor_view::*;
use obsidian_ui_inspect::{InspectableResource, Inspector};

#[derive(Resource, Debug, Reflect, Clone, Default)]
pub struct TestStruct {
    pub selected: bool,
    pub scale: f32,
    pub color: Srgba,
    pub position: Vec3,

    pub unlit: Option<bool>,
    pub roughness: Option<f32>,
    pub metalness: Option<f32>,
    pub factors: Vec<f32>,
}

pub struct ResourcePropertyInspector<T: Resource> {
    marker: std::marker::PhantomData<T>,
}

impl<T: Resource> ResourcePropertyInspector<T> {
    pub fn new() -> Self {
        Self {
            marker: std::marker::PhantomData,
        }
    }
}

impl<T: Resource + Reflect> ViewTemplate for ResourcePropertyInspector<T> {
    fn create(&self, _cx: &mut Cx) -> impl IntoView {
        Inspector::new(Arc::<InspectableResource<T>>::default())
    }
}
