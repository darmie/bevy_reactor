use bevy::{prelude::*, ui};
use bevy_reactor_core::*;
use bevy_reactor_style::*;
use bevy_reactor_view::*;
use obsidian_ui::{
    colors,
    controls::{Button, Icon, Spacer},
    typography,
};

use crate::InspectableField;

fn style_field_label(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .align_items(ui::AlignItems::Center)
        .justify_content(ui::JustifyContent::FlexStart)
        .font_size(16)
        .min_width(64)
        .color(colors::DIM);
}

/// Label for editable struct field in an inspector.
pub struct FieldLabel {
    /// The content of the label.
    pub field: InspectableField,
}

impl ViewTemplate for FieldLabel {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let can_remove = self.field.can_remove();
        let field = self.field.clone();
        let remove = cx.create_callback(move |cx, _| {
            field.remove(cx);
        });
        Element::<NodeBundle>::new()
            .style((typography::text_default, style_field_label))
            .children((
                self.field.name(),
                Cond::new(
                    move |_cx| can_remove,
                    move || {
                        Button::new()
                            .children(
                                Icon::new("obsidian_ui://icons/close.png")
                                    .size(Vec2::splat(10.))
                                    .color(Color::from(colors::DIM))
                                    .style(|ss: &mut StyleBuilder| {
                                        ss.margin((4, 0));
                                    }),
                            )
                            .minimal(true)
                            .on_click(remove)
                    },
                    || (),
                ),
                Spacer,
            ))
    }
}

fn style_field_label_wide(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .align_items(ui::AlignItems::Center)
        .justify_content(ui::JustifyContent::FlexStart)
        .grid_column_span(2)
        .font_size(16)
        .min_width(64)
        .color(colors::DIM);
}

/// Label for editable struct field in an inspector.
pub struct FieldLabelWide {
    /// The content of the label.
    pub field: InspectableField,
    /// Additional buttons in the label.
    pub buttons: Option<ViewRef>,
}

impl ViewTemplate for FieldLabelWide {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let can_remove = self.field.can_remove();
        let field = self.field.clone();
        let remove = cx.create_callback(move |cx, _| {
            field.remove(cx);
        });
        Element::<NodeBundle>::new()
            .style((typography::text_default, style_field_label_wide))
            .children((
                self.field.name(),
                Spacer,
                self.buttons.clone(),
                Cond::new(
                    move |_cx| can_remove,
                    move || {
                        Button::new()
                            .children(
                                Icon::new("obsidian_ui://icons/close.png")
                                    .size(Vec2::splat(10.))
                                    .color(Color::from(colors::DIM))
                                    .style(|ss: &mut StyleBuilder| {
                                        ss.margin((4, 0));
                                    }),
                            )
                            .minimal(true)
                            .on_click(remove)
                    },
                    || (),
                ),
            ))
    }
}
