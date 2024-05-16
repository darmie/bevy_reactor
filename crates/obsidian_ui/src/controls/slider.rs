use bevy::{
    color::{LinearRgba, Luminance},
    prelude::*,
    ui,
};
use bevy_mod_picking::prelude::*;
use bevy_reactor_core::{Callback, CreateHoverSignal, Cx, EffectTarget, IntoSignal, Mutable, RunContextSetup, RunContextWrite, Signal};
use bevy_reactor_style::*;
use bevy_reactor_view::{text_computed, Cond, Element, IntoView, ParentView, ViewTemplate, Fragment};

use crate::{colors, materials::SliderRectMaterial, RoundedCorners};

#[derive(Clone, PartialEq, Default, Copy)]
enum DragType {
    #[default]
    None = 0,
    Dragging,
    HoldDecrement,
    HoldIncrement,
}

#[derive(Clone, PartialEq, Default, Copy)]
struct DragState {
    dragging: DragType,
    offset: f32,
}

fn style_slider(ss: &mut StyleBuilder) {
    ss.min_width(64).height(20);
}

fn style_overlay(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .align_items(ui::AlignItems::Center)
        .position(ui::PositionType::Absolute)
        .left(0)
        .top(0)
        .bottom(0)
        .right(0);
}

fn style_button(ss: &mut StyleBuilder) {
    ss.width(12)
        .height(ui::Val::Percent(100.))
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .align_items(ui::AlignItems::Center)
        .justify_content(ui::JustifyContent::Center)
        .font("obsidian_ui://fonts/Open_Sans/static/OpenSans-Regular.ttf")
        .font_size(16);
}

fn style_button_icon(ss: &mut StyleBuilder) {
    ss.height(16).width(12).pointer_events(false);
}

fn style_button_icon_left(ss: &mut StyleBuilder) {
    ss.background_image("obsidian_ui://icons/chevron_left.png");
}

fn style_button_icon_right(ss: &mut StyleBuilder) {
    ss.background_image("obsidian_ui://icons/chevron_right.png");
}

fn style_label(ss: &mut StyleBuilder) {
    ss.flex_grow(1.)
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .align_items(ui::AlignItems::Center)
        .justify_content(ui::JustifyContent::Center)
        .height(ui::Val::Percent(100.))
        // .border_color(colors::DESTRUCTIVE)
        .font("obsidian_ui://fonts/Open_Sans/static/OpenSans-Medium.ttf")
        .font_size(16)
        .padding((6, 0));
}

fn style_label_spacer(ss: &mut StyleBuilder) {
    ss.flex_grow(1.);
}

/// Horizontal slider widget
pub struct Slider {
    /// Current slider value.
    pub value: Signal<f32>,

    /// Minimum slider value.
    pub min: Signal<f32>,

    /// Maximum slider value.
    pub max: Signal<f32>,

    /// Number of decimal places to round to (0 = integer).
    pub precision: usize,

    /// Amount to increment when using arrow buttons.
    pub step: f32,

    /// Whether the slider is disabled.
    pub disabled: Signal<bool>,

    /// Signal which returns the value formatted as a string. It `None`, then a default
    /// formatter will be used.
    pub formatted_value: Option<Signal<String>>,

    /// Optional label to be displayed inside the slider.
    pub label: Option<String>,

    /// Style handle for slider root element.
    pub style: StyleHandle,

    /// Callback called when value changes
    pub on_change: Option<Callback<f32>>,
}

impl Slider {
    /// Create a new slider.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the current slider value.
    pub fn value(mut self, value: impl IntoSignal<f32>) -> Self {
        self.value = value.into_signal();
        self
    }

    /// Set the minimum slider value.
    pub fn min(mut self, min: impl IntoSignal<f32>) -> Self {
        self.min = min.into_signal();
        self
    }

    /// Set the maximum slider value.
    pub fn max(mut self, max: impl IntoSignal<f32>) -> Self {
        self.max = max.into_signal();
        self
    }

    /// Set the number of decimal places to round to (0 = integer).
    pub fn precision(mut self, precision: usize) -> Self {
        self.precision = precision;
        self
    }

    /// Set the amount to increment when using arrow buttons.
    pub fn step(mut self, step: f32) -> Self {
        self.step = step;
        self
    }

    /// Set whether the slider is disabled.
    pub fn disabled(mut self, disabled: impl IntoSignal<bool>) -> Self {
        self.disabled = disabled.into_signal();
        self
    }

    /// Set the signal which returns the value formatted as a string. If `None`, then a default
    /// formatter will be used.
    pub fn formatted_value(mut self, formatted_value: impl IntoSignal<String>) -> Self {
        self.formatted_value = Some(formatted_value.into_signal());
        self
    }

    /// Set the optional label to be displayed inside the slider.
    pub fn label(mut self, label: String) -> Self {
        self.label = Some(label);
        self
    }

    /// Set the style handle for the slider root element.
    pub fn style<S: StyleTuple + 'static>(mut self, style: S) -> Self {
        self.style = style.into_handle();
        self
    }

    /// Set the callback called when value changes.
    pub fn on_change(mut self, on_change: Callback<f32>) -> Self {
        self.on_change = Some(on_change);
        self
    }
}

impl Default for Slider {
    fn default() -> Self {
        Self {
            value: Signal::Constant(0.),
            min: Signal::Constant(0.),
            max: Signal::Constant(1.),
            precision: 0,
            step: 1.,
            disabled: Signal::Constant(false),
            formatted_value: None,
            style: StyleHandle::default(),
            label: None,
            on_change: None,
        }
    }
}

impl ViewTemplate for Slider {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let slider_id = cx.create_entity();
        let hovering = cx.create_hover_signal(slider_id);
        let drag_state = cx.create_mutable::<DragState>(DragState::default());

        // Pain point: Need to capture all props for closures.
        let min = self.min;
        let max = self.max;
        let value = self.value;
        let precision = self.precision;
        let step = self.step;
        let on_change = self.on_change;

        let mut ui_materials = cx
            .world_mut()
            .get_resource_mut::<Assets<SliderRectMaterial>>()
            .unwrap();
        let material = ui_materials.add(SliderRectMaterial {
            color_lo: LinearRgba::from(colors::U1).to_vec4(),
            color_hi: LinearRgba::from(colors::U3).to_vec4(),
            value: 0.5,
            radius: RoundedCorners::All.to_vec(4.),
        });

        Element::<MaterialNodeBundle<SliderRectMaterial>>::for_entity(slider_id)
            .style((style_slider, self.style.clone()))
            .insert((
                material.clone(),
                On::<Pointer<DragStart>>::run(move |world: &mut World| {
                    // Save initial value to use as drag offset.
                    let mut event = world
                        .get_resource_mut::<ListenerInput<Pointer<DragStart>>>()
                        .unwrap();
                    event.stop_propagation();
                    drag_state.set(
                        world,
                        DragState {
                            dragging: DragType::Dragging,
                            offset: value.get(world),
                        },
                    );
                }),
                On::<Pointer<DragEnd>>::run(move |world: &mut World| {
                    let ds = drag_state.get(world);
                    if ds.dragging == DragType::Dragging {
                        drag_state.set(
                            world,
                            DragState {
                                dragging: DragType::None,
                                offset: value.get(world),
                            },
                        );
                    }
                }),
                On::<Pointer<Drag>>::run(move |world: &mut World| {
                    let ds = drag_state.get(world);
                    if ds.dragging == DragType::Dragging {
                        let event = world
                            .get_resource::<ListenerInput<Pointer<Drag>>>()
                            .unwrap();
                        let ent = world.entity(slider_id);
                        let node = ent.get::<Node>();
                        let transform = ent.get::<GlobalTransform>();
                        if let (Some(node), Some(transform)) = (node, transform) {
                            // Measure node width and slider value.
                            let slider_width = node.logical_rect(transform).width();
                            let min = min.get(world);
                            let max = max.get(world);
                            let range = max - min;
                            let new_value = if range > 0. {
                                ds.offset + (event.distance.x * range) / slider_width
                            } else {
                                min + range * 0.5
                            };
                            let rounding = f32::powi(10., precision as i32);
                            let new_value = (new_value * rounding).round() / rounding;
                            if let Some(on_change) = on_change {
                                world.run_callback(on_change, new_value.clamp(min, max));
                            }
                        }
                    }
                }),
            ))
            .create_effect(move |cx, _ent| {
                let min = min.get(cx);
                let max = max.get(cx);
                let value = value.get(cx);
                let pos = if max > min {
                    (value - min) / (max - min)
                } else {
                    0.
                };

                let mut ui_materials = cx
                    .world_mut()
                    .get_resource_mut::<Assets<SliderRectMaterial>>()
                    .unwrap();
                let material = ui_materials.get_mut(material.id()).unwrap();
                material.value = pos;
            })
            .children((Element::<NodeBundle>::new()
                .named("Slider")
                .style(style_overlay)
                .children((
                    SliderButton {
                        value,
                        min,
                        max,
                        step: -step,
                        hovering,
                        on_change,
                        drag_state,
                    },
                    Element::<NodeBundle>::new().style(style_label).children((
                        Cond::new(
                            {
                                let label = self.label.clone();
                                move |_cx| label.is_some()
                            },
                            {
                                let label = self.label.clone();
                                move || {
                                    Fragment::new((
                                        label.clone().unwrap(),
                                        Element::<NodeBundle>::new().style(style_label_spacer),
                                    ))
                                }
                            },
                            || (),
                        ),
                        text_computed({
                            move |cx| {
                                let value = value.get(cx);
                                format!("{:.*}", precision, value)
                            }
                        }),
                    )),
                    SliderButton {
                        value,
                        min,
                        max,
                        step,
                        hovering,
                        on_change,
                        drag_state,
                    },
                )),))
    }
}

struct SliderButton {
    value: Signal<f32>,
    min: Signal<f32>,
    max: Signal<f32>,
    step: f32,
    hovering: Signal<bool>,
    on_change: Option<Callback<f32>>,
    drag_state: Mutable<DragState>,
}

impl ViewTemplate for SliderButton {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let button_id = cx.create_entity();
        let button_hovering = cx.create_hover_signal(button_id);
        let hovering = self.hovering;
        let min = self.min;
        let max = self.max;
        let value = self.value;
        let step = self.step;
        let on_change = self.on_change;
        let drag_state = self.drag_state;
        let drag_type = if step > 0.0 {
            DragType::HoldIncrement
        } else {
            DragType::HoldDecrement
        };

        Element::<NodeBundle>::for_entity(button_id)
            .style(style_button)
            .insert((
                On::<Pointer<DragStart>>::run(move |world: &mut World| {
                    let mut event = world
                        .get_resource_mut::<ListenerInput<Pointer<DragStart>>>()
                        .unwrap();
                    event.stop_propagation();
                    drag_state.set(
                        world,
                        DragState {
                            dragging: drag_type,
                            offset: value.get(world),
                        },
                    );
                    let min = min.get(world);
                    let max = max.get(world);
                    let value = value.get(world) + step;
                    if let Some(on_change) = on_change {
                        world.run_callback(on_change, value.clamp(min, max));
                    }
                }),
                On::<Pointer<DragEnd>>::run(move |world: &mut World| {
                    let mut event = world
                        .get_resource_mut::<ListenerInput<Pointer<DragEnd>>>()
                        .unwrap();
                    event.stop_propagation();
                    drag_state.set(
                        world,
                        DragState {
                            dragging: DragType::None,
                            offset: value.get(world),
                        },
                    );
                }),
                On::<Pointer<DragEnter>>::run(move |world: &mut World| {
                    let ds = drag_state.get(world);
                    let mut event = world
                        .get_resource_mut::<ListenerInput<Pointer<DragEnter>>>()
                        .unwrap();
                    if ds.dragging == DragType::None {
                        event.stop_propagation();
                        drag_state.set(
                            world,
                            DragState {
                                dragging: drag_type,
                                offset: value.get(world),
                            },
                        );
                    }
                }),
                On::<Pointer<DragLeave>>::run(move |world: &mut World| {
                    let ds = drag_state.get(world);
                    if ds.dragging == drag_type {
                        let mut event = world
                            .get_resource_mut::<ListenerInput<Pointer<DragLeave>>>()
                            .unwrap();
                        event.stop_propagation();
                        drag_state.set(
                            world,
                            DragState {
                                dragging: DragType::None,
                                offset: value.get(world),
                            },
                        );
                    }
                }),
            ))
            .children(
                Element::<NodeBundle>::new()
                    .style((
                        style_button_icon,
                        if step > 0.0 {
                            style_button_icon_right
                        } else {
                            style_button_icon_left
                        },
                    ))
                    .create_effect(move |cx, ent| {
                        let ds = drag_state.get(cx);
                        let is_hovering = hovering.get(cx) && step != 0.0;
                        let is_hovering_inc = button_hovering.get(cx);
                        let color = match (ds.dragging, is_hovering, is_hovering_inc) {
                            (DragType::HoldIncrement, _, _) if step > 0.0 => colors::FOREGROUND,
                            (DragType::HoldDecrement, _, _) if step < 0.0 => colors::FOREGROUND,
                            (DragType::Dragging, _, _) => colors::TRANSPARENT,
                            (_, true, true) => colors::U4.lighter(0.1),
                            (_, true, false) => colors::U4,
                            _ => colors::TRANSPARENT,
                        };
                        let mut bg = cx.world_mut().get_mut::<UiImage>(ent).unwrap();
                        bg.color = color.into();
                    }),
            )
    }
}
