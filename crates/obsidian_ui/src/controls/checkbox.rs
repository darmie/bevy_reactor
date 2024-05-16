use bevy::{
    a11y::{
        accesskit::{NodeBuilder, Role},
        AccessibilityNode, Focus,
    },
    color::{LinearRgba, Luminance},
    prelude::*,
    ui,
};
use bevy_mod_picking::{events::PointerCancel, prelude::*};
use bevy_reactor_style::*;
use bevy_reactor_core::{Callback, CreateHoverSignal, Cx, EffectTarget, IntoSignal, RunContextSetup, RunContextWrite, Signal};
use bevy_reactor_view::{ChildArray, ChildViewTuple, Element, IntoView, ParentView, ViewTemplate, Cond};

use crate::{
    colors,
    focus::{KeyPressEvent, TabIndex},
    hooks::CreateFocusSignal,
};

fn style_checkbox(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .justify_content(ui::JustifyContent::Center)
        .align_items(ui::AlignItems::Center)
        .align_content(ui::AlignContent::Center)
        .gap(4)
        .color(colors::FOREGROUND);
}

fn style_checkbox_border(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .width(16)
        .height(16)
        .border_radius(3.0);
}

fn style_checkbox_inner(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .background_image("obsidian_ui://textures/checkmark.png")
        // .background_color(colors::FOREGROUND)
        .position(ui::PositionType::Absolute)
        .left(2)
        .top(2)
        .width(12)
        .height(12);
}

fn style_checkbox_label(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .justify_content(ui::JustifyContent::FlexStart)
        .align_items(ui::AlignItems::Center)
        .color(colors::FOREGROUND);
}

/// A checkbox widget.
#[derive(Default)]
pub struct Checkbox {
    /// Whether the checkbox is checked.
    pub checked: Signal<bool>,

    /// Whether the checkbox is disabled.
    pub disabled: Signal<bool>,

    /// The content to display inside the button.
    pub label: ChildArray,

    /// Additional styles to be applied to the button.
    pub style: StyleHandle,

    /// Callback called when clicked
    pub on_change: Option<Callback<bool>>,

    /// The tab index of the checkbox (default 0).
    pub tab_index: i32,
}

impl Checkbox {
    /// Create a new checkbox.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the checked state of the checkbox.
    pub fn checked(mut self, checked: impl IntoSignal<bool>) -> Self {
        self.checked = checked.into_signal();
        self
    }

    /// Set the disabled state of the checkbox.
    pub fn disabled(mut self, disabled: impl IntoSignal<bool>) -> Self {
        self.disabled = disabled.into_signal();
        self
    }

    /// Set the label of the checkbox.
    pub fn label<V: ChildViewTuple>(mut self, label: V) -> Self {
        self.label = label.to_child_array();
        self
    }

    /// Set the style of the checkbox.
    pub fn style<S: StyleTuple + 'static>(mut self, style: S) -> Self {
        self.style = style.into_handle();
        self
    }

    /// Set the on_change callback of the checkbox.
    pub fn on_change(mut self, on_change: Callback<bool>) -> Self {
        self.on_change = Some(on_change);
        self
    }

    /// Set the tab index of the checkbox.
    pub fn tab_index(mut self, tab_index: i32) -> Self {
        self.tab_index = tab_index;
        self
    }
}

impl ViewTemplate for Checkbox {
    /// Construct a checkbox widget.
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let id = cx.create_entity();
        let pressed = cx.create_mutable::<bool>(false);
        let hovering = cx.create_hover_signal(id);
        let focused = cx.create_focus_visible_signal(id);

        let disabled = self.disabled;
        let checked = self.checked;

        Element::<NodeBundle>::for_entity(id)
            .named("Checkbox")
            .style((style_checkbox, self.style.clone()))
            .insert((
                TabIndex(self.tab_index),
                AccessibilityNode::from(NodeBuilder::new(Role::CheckBox)),
                {
                    let on_change = self.on_change;
                    On::<Pointer<Click>>::run(move |world: &mut World| {
                        let mut focus = world.get_resource_mut::<Focus>().unwrap();
                        focus.0 = Some(id);
                        if !disabled.get(world) {
                            let next_checked = checked.get(world);
                            if let Some(on_click) = on_change {
                                world.run_callback(on_click, !next_checked);
                            }
                        }
                    })
                },
                On::<Pointer<DragStart>>::run(move |world: &mut World| {
                    if !disabled.get(world) {
                        pressed.set(world, true);
                    }
                }),
                On::<Pointer<DragEnd>>::run(move |world: &mut World| {
                    if !disabled.get(world) {
                        pressed.set(world, false);
                    }
                }),
                On::<Pointer<DragEnter>>::run(move |world: &mut World| {
                    if !disabled.get(world) {
                        pressed.set(world, true);
                    }
                }),
                On::<Pointer<DragLeave>>::run(move |world: &mut World| {
                    if !disabled.get(world) {
                        pressed.set(world, false);
                    }
                }),
                On::<Pointer<PointerCancel>>::run(move |world: &mut World| {
                    println!("PointerCancel");
                    if !disabled.get(world) {
                        pressed.set(world, false);
                    }
                }),
                On::<KeyPressEvent>::run({
                    let on_change = self.on_change;
                    move |world: &mut World| {
                        if !disabled.get(world) {
                            let mut event = world
                                .get_resource_mut::<ListenerInput<KeyPressEvent>>()
                                .unwrap();
                            if !event.repeat
                                && (event.key_code == KeyCode::Enter
                                    || event.key_code == KeyCode::Space)
                            {
                                event.stop_propagation();
                                let next_checked = checked.get(world);
                                if let Some(on_click) = on_change {
                                    world.run_callback(on_click, !next_checked);
                                }
                            }
                        }
                    }
                }),
            ))
            .children((
                Element::<NodeBundle>::new()
                    .named("Checkbox::Border")
                    .style(style_checkbox_border)
                    .create_effect(move |cx, ent| {
                        let is_checked = checked.get(cx);
                        let is_pressed = pressed.get(cx);
                        let is_hovering = hovering.get(cx);
                        let color = match (is_checked, is_pressed, is_hovering) {
                            (true, true, _) => colors::ACCENT.darker(0.1),
                            (true, false, true) => colors::ACCENT.darker(0.15),
                            (true, _, _) => colors::ACCENT.darker(0.2),
                            (false, true, _) => colors::U1.lighter(0.005),
                            (false, false, true) => colors::U1.lighter(0.002),
                            (false, false, false) => colors::U1,
                        };
                        let mut bg = cx.world_mut().get_mut::<BackgroundColor>(ent).unwrap();
                        bg.0 = LinearRgba::from(color).into();
                    })
                    .create_effect(move |cx, entt| {
                        let is_focused = focused.get(cx);
                        let mut entt = cx.world_mut().entity_mut(entt);
                        match is_focused {
                            true => {
                                entt.insert(Outline {
                                    color: colors::FOCUS.into(),
                                    offset: ui::Val::Px(2.0),
                                    width: ui::Val::Px(2.0),
                                });
                            }
                            false => {
                                entt.remove::<Outline>();
                            }
                        };
                    })
                    .children(Cond::new(
                        move |cx| checked.get(cx),
                        move || Element::<NodeBundle>::new().style(style_checkbox_inner),
                        || (),
                    )),
                Element::<NodeBundle>::new()
                    .style(style_checkbox_label)
                    .children(self.label.clone()),
            ))
    }
}
