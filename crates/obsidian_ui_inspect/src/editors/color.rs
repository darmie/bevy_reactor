use bevy::{prelude::*, ui};
use bevy_reactor_core::*;
use bevy_reactor_style::*;
use bevy_reactor_view::*;
use obsidian_ui::{
    colors,
    controls::{Icon, MenuButton, MenuPopup, Spacer, Swatch},
    floating::{FloatAlign, FloatSide},
    size::Size,
};

fn style_field(ss: &mut StyleBuilder) {
    ss.flex_direction(ui::FlexDirection::Row)
        .align_items(ui::AlignItems::Center)
        .justify_content(ui::JustifyContent::FlexStart);
}

fn style_swatch(ss: &mut StyleBuilder) {
    ss.width(16).height(16).margin_right(4);
}

fn style_menu_icon(ss: &mut StyleBuilder) {
    ss.margin((2, 0));
}

use crate::{
    templates::{
        color_edit::{ColorEdit, ColorEditState, ColorMode},
        field_label::FieldLabel,
    },
    InspectableField,
};

pub struct FieldEditSrgba {
    pub(crate) field: InspectableField,
}

impl ViewTemplate for FieldEditSrgba {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let field = self.field.clone();
        let value = cx.create_memo(move |cx| {
            if let Some(value) = field.reflect(cx) {
                if value.is::<Srgba>() {
                    return *value.downcast_ref::<Srgba>().unwrap();
                }
            }
            Srgba::NONE
        });

        let state = cx.create_mutable(ColorEditState {
            mode: ColorMode::Rgb,
            rgb: Srgba::default(),
            hsl: Hsla::default(),
        });

        let field = self.field.clone();
        cx.create_effect(move |cx| {
            let next_state = state.get(cx);
            if let Some(reflect) = field.reflect(cx) {
                let value = *reflect.downcast_ref::<Srgba>().unwrap();
                if value != next_state.rgb {
                    field.set_value(cx, &next_state.rgb);
                }
            }
        });

        Fragment::new((
            FieldLabel {
                field: self.field.clone(),
            },
            Element::<NodeBundle>::new().style(style_field).children((
                Swatch::new(value).style(style_swatch),
                text_computed(move |cx| {
                    let value = value.get(cx);
                    value.to_hex()
                }),
                Spacer,
                MenuButton::new()
                    .children(
                        Icon::new("obsidian_ui://icons/tune.png")
                            .size(Vec2::splat(16.0))
                            .style(style_menu_icon)
                            .color(Color::from(colors::DIM)),
                    )
                    .popup(
                        MenuPopup::new()
                            .side(FloatSide::Right)
                            .align(FloatAlign::Start)
                            .children(ColorEdit::new(
                                state,
                                cx.create_callback(move |cx, st: ColorEditState| {
                                    state.set(cx, st);
                                }),
                            )),
                    )
                    .size(Size::Xxs)
                    .minimal(true)
                    .no_caret(true),
            )),
        ))
    }
}
