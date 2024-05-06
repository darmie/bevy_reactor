use bevy::{asset::AssetPath, prelude::*};
use bevy_reactor::*;

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
    fn create(&self, _cx: &mut Cx) -> impl Into<ViewRef> {
        let color = self.color;
        let icon = self.icon.clone();
        let size = self.size;

        Element::<NodeBundle>::new()
            .with_styles((
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
