use bevy::{color::Srgba, prelude::*, ui};
use bevy_mod_picking::prelude::*;
use bevy_reactor_core::{Callback, Cx, IntoSignal, RunContextSetup, RunContextWrite, Signal, EffectTarget};
use bevy_reactor_style::{StyleBuilder, StyleBuilderBackground, StyleBuilderLayout, StyleHandle, StyleTuple, WithStyles};
use bevy_reactor_view::{Element, IntoView, ParentView, ViewTemplate};

use crate::materials::GradientRectMaterial;

const THUMB_WIDTH: f32 = 12.;

/// Struct representing a sequence of color stops, evenly spaced. Up to 8 stops are supported.
#[derive(Debug, Copy, Clone)]
pub struct ColorGradient {
    /// Number of color stops.
    pub num_colors: usize,

    /// Array of color stops.
    pub colors: [Srgba; 8],
}

impl ColorGradient {
    /// Construct a new color gradient from an array of colors.
    pub fn new(colors: &[Srgba]) -> Self {
        assert!(colors.len() <= 8);
        let mut result = Self {
            num_colors: colors.len(),
            colors: [Srgba::default(); 8],
        };
        for (i, color) in colors.iter().enumerate() {
            result.colors[i] = *color;
        }
        result
    }

    /// Return the first color in the gradient, if any.
    pub fn first(&self) -> Option<Srgba> {
        if self.num_colors > 0 {
            Some(self.colors[0])
        } else {
            None
        }
    }

    /// Return the last color in the gradient, if any.
    pub fn last(&self) -> Option<Srgba> {
        if self.num_colors > 0 {
            Some(self.colors[self.num_colors - 1])
        } else {
            None
        }
    }

    /// Return the number of color stops in the gradient.
    pub fn len(&self) -> usize {
        self.num_colors
    }

    /// Check if the gradient is empty.
    pub fn is_empty(&self) -> bool {
        self.num_colors == 0
    }
}

impl Default for ColorGradient {
    fn default() -> Self {
        Self {
            num_colors: 1,
            colors: [Srgba::BLACK; 8],
        }
    }
}

#[derive(Clone, PartialEq, Default, Copy)]
struct DragState {
    dragging: bool,
    offset: f32,
}

fn style_slider(ss: &mut StyleBuilder) {
    ss.min_width(32)
        .height(14)
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .align_items(ui::AlignItems::Stretch);
}

fn style_gradient(ss: &mut StyleBuilder) {
    ss.flex_grow(1.);
}

fn style_track(ss: &mut StyleBuilder) {
    ss.position(ui::PositionType::Absolute)
        .top(1)
        .bottom(1)
        .left(1)
        .right(THUMB_WIDTH + 1.);
}

fn style_thumb(ss: &mut StyleBuilder) {
    ss.background_image("obsidian_ui://textures/gradient_thumb.png")
        .position(ui::PositionType::Absolute)
        .top(0)
        .bottom(0)
        .width(THUMB_WIDTH);
}

/// Horizontal slider widget that displays a gradient bar and a draggable button.
pub struct GradientSlider {
    /// Gradient to display.
    pub gradient: Signal<ColorGradient>,

    /// Current slider value.
    pub value: Signal<f32>,

    /// Minimum slider value.
    pub min: Signal<f32>,

    /// Maximum slider value.
    pub max: Signal<f32>,

    /// Number of decimal places to round to (0 = integer).
    pub precision: usize,

    /// Whether the slider is disabled.
    pub disabled: Signal<bool>,

    /// Style handle for slider root element.
    pub style: StyleHandle,

    /// Callback called when value changes
    pub on_change: Option<Callback<f32>>,
}

impl GradientSlider {
    /// Create a new gradient slider.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the gradient to display.
    pub fn gradient(mut self, gradient: impl IntoSignal<ColorGradient>) -> Self {
        self.gradient = gradient.into_signal();
        self
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

    /// Set whether the slider is disabled.
    pub fn disabled(mut self, disabled: impl IntoSignal<bool>) -> Self {
        self.disabled = disabled.into_signal();
        self
    }

    /// Set the style handle for the slider root element.
    pub fn style<S: StyleTuple + 'static>(mut self, style: S) -> Self {
        self.style = style.into_handle();
        self
    }

    /// Set the callback called when the value changes.
    pub fn on_change(mut self, on_change: Callback<f32>) -> Self {
        self.on_change = Some(on_change);
        self
    }
}

impl Default for GradientSlider {
    fn default() -> Self {
        Self {
            gradient: Signal::Constant(ColorGradient::default()),
            value: Signal::Constant(0.),
            min: Signal::Constant(0.),
            max: Signal::Constant(1.),
            precision: 0,
            disabled: Signal::Constant(false),
            style: StyleHandle::default(),
            on_change: None,
        }
    }
}

impl ViewTemplate for GradientSlider {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let slider_id = cx.create_entity();
        // let hovering = cx.create_hover_signal(slider_id);
        let drag_state = cx.create_mutable::<DragState>(DragState::default());

        // Pain point: Need to capture all props for closures.
        let min = self.min;
        let max = self.max;
        let value = self.value;
        let precision = self.precision;
        let on_change = self.on_change;

        // This should really be an effect.
        let color_stops: Signal<(usize, [Vec4; 8])> = {
            let gradient = self.gradient;
            cx.create_derived(move |cc| {
                gradient.map(cc, |g| {
                    let mut result: [Vec4; 8] = [Vec4::default(); 8];
                    let num_color_stops = g.len();
                    for (i, color) in g.colors[0..num_color_stops].iter().enumerate() {
                        // Note that we do *not* convert to linear here, because interpolating
                        // linear looks bad. That gets done in the shader.
                        result[i] = Vec4::new(color.red, color.green, color.blue, color.alpha);
                    }
                    (g.len(), result)
                })
            })
        };

        let mut gradient_material_assets = cx
            .world_mut()
            .get_resource_mut::<Assets<GradientRectMaterial>>()
            .unwrap();
        let gradient_material = gradient_material_assets.add(GradientRectMaterial {
            color_stops: [Srgba::default().to_vec4(); 8],
            num_color_stops: 2,
            cap_size: THUMB_WIDTH * 0.5,
        });

        // Effect to update the material handle.
        cx.create_effect({
            let material = gradient_material.clone();
            move |cx| {
                let (num_color_stops, color_stops) = color_stops.get(cx);
                let mut ui_materials = cx
                    .world_mut()
                    .get_resource_mut::<Assets<GradientRectMaterial>>()
                    .unwrap();
                let material = ui_materials.get_mut(material.id()).unwrap();
                material.num_color_stops = num_color_stops as i32;
                material.color_stops = color_stops;
            }
        });

        Element::<NodeBundle>::for_entity(slider_id)
            .named("GradientSlider")
            .style((style_slider, self.style.clone()))
            .insert((
                On::<Pointer<Down>>::run(move |world: &mut World| {
                    let min = min.get(world);
                    let max = max.get(world);
                    let mut event = world
                        .get_resource_mut::<ListenerInput<Pointer<Down>>>()
                        .unwrap();
                    event.stop_propagation();
                    let hit_x = event.pointer_location.position.x;
                    let ent = world.entity(slider_id);
                    let node = ent.get::<Node>();
                    let transform = ent.get::<GlobalTransform>();
                    if let (Some(node), Some(transform)) = (node, transform) {
                        // If not clicking on thumb, then snap thumb to new location.
                        let rect = node.logical_rect(transform);
                        let slider_width = rect.width() - THUMB_WIDTH;
                        let range = max - min;
                        let pointer_pos = hit_x - rect.min.x - THUMB_WIDTH / 2.;
                        let thumb_pos =
                            value.get(world) - min * slider_width / range + THUMB_WIDTH / 2.;
                        if range > 0. && (pointer_pos - thumb_pos).abs() >= THUMB_WIDTH / 2. {
                            let new_value = min + (pointer_pos * range) / slider_width;
                            if let Some(on_change) = on_change {
                                world.run_callback(on_change, new_value.clamp(min, max));
                            }
                        };
                    }
                }),
                On::<Pointer<DragStart>>::run(move |world: &mut World| {
                    // Save initial value to use as drag offset.
                    let mut event = world
                        .get_resource_mut::<ListenerInput<Pointer<DragStart>>>()
                        .unwrap();
                    event.stop_propagation();
                    drag_state.set(
                        world,
                        DragState {
                            dragging: true,
                            offset: value.get(world),
                        },
                    );
                }),
                On::<Pointer<DragEnd>>::run(move |world: &mut World| {
                    let ds = drag_state.get(world);
                    if ds.dragging {
                        drag_state.set(
                            world,
                            DragState {
                                dragging: false,
                                offset: value.get(world),
                            },
                        );
                    }
                }),
                On::<Pointer<Drag>>::run(move |world: &mut World| {
                    let ds = drag_state.get(world);
                    if ds.dragging {
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
            .children((
                Element::<MaterialNodeBundle<GradientRectMaterial>>::new()
                    .insert(gradient_material.clone())
                    .style(style_gradient),
                Element::<NodeBundle>::new()
                    .named("GradientSlider::Track")
                    .style(style_track)
                    .children(
                        Element::<NodeBundle>::new()
                            .named("GradientSlider::Thumb")
                            .style(style_thumb)
                            .create_effect(move |cx, ent| {
                                let min = min.get(cx);
                                let max = max.get(cx);
                                let value = value.get(cx);
                                let percent = if max > min {
                                    ((value - min) / (max - min)).clamp(0., 1.)
                                } else {
                                    0.
                                };

                                let mut style = cx.world_mut().get_mut::<Style>(ent).unwrap();
                                style.left = ui::Val::Percent(percent * 100.);
                            }),
                    ),
            ))
    }
}
