use bevy::{asset::AssetPath, prelude::*};
use bevy_reactor_core::{Cx,IntoSignal, Signal};
use bevy_reactor_style::{StyleBuilder, StyleBuilderBackground, StyleBuilderLayout, StyleHandle, StyleTuple, WithStyles};
use bevy_reactor_view::{Element, IntoView, ViewTemplate};
use bevy_reactor_core::EffectTarget;

use crate::colors;

/// Control that displays an icon.
pub struct Icon {
    /// Asset path for the icon
    pub icon: String,

    /// Size of the icon in pixels.
    pub size: Vec2,

    /// Color of the icon.
    pub color: Signal<Color>,

    /// Additional styles to apply to the icon
    pub style: StyleHandle,
}

impl Icon {
    /// Create a new icon.
    pub fn new(icon: &str) -> Self {
        Self {
            icon: icon.to_string(),
            ..default()
        }
    }

    /// Set the size of the icon.
    pub fn size(mut self, size: Vec2) -> Self {
        self.size = size;
        self
    }

    /// Set the color of the icon.
    pub fn color(mut self, color: impl IntoSignal<Color>) -> Self {
        self.color = color.into_signal();
        self
    }

    /// Set the style of the icon.
    pub fn style<S: StyleTuple + 'static>(mut self, style: S) -> Self {
        self.style = style.into_handle();
        self
    }
}

impl Default for Icon {
    fn default() -> Self {
        Self {
            icon: "".to_string(),
            size: Vec2::splat(12.0),
            color: Signal::Constant(colors::FOREGROUND.into()),
            style: StyleHandle::default(),
        }
    }
}

impl ViewTemplate for Icon {
    fn create(&self, _cx: &mut Cx) -> impl IntoView {
        let color = self.color;
        let icon = self.icon.clone();
        let size = self.size;

        Element::<NodeBundle>::new()
            .style((
                move |sb: &mut StyleBuilder| {
                    sb.width(size.x)
                        .height(size.y)
                        .background_image(AssetPath::parse(&icon));
                },
                self.style.clone(),
            ))
            .create_effect(move |cx, ent| {
                let color = color.get(cx);
                let mut ent = cx.world_mut().entity_mut(ent);
                let mut uii = ent.get_mut::<UiImage>().unwrap();
                uii.color = color;
            })
    }
}
