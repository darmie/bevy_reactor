use bevy::{color::Luminance, prelude::*, ui};
use bevy_mod_picking::{events::PointerCancel, prelude::*};
use bevy_reactor_core::{Callback, CreateHoverSignal, Cx, EffectTarget, IntoSignal, RunContextSetup, RunContextWrite, Signal};
use bevy_reactor_style::*;
use bevy_reactor_view::{Element, IntoView, ParentView, ViewTemplate};


use crate::colors;

/// The direction of the splitter. Represents the direction of the bar, not the items being split.
#[derive(Clone, PartialEq, Default)]
pub enum SplitterDirection {
    /// The splitter bar runs horizontally, and splits the items above and below it.
    Horizontal,

    /// The splitter bar runs vertically, and splits the items to the left and right of it.
    #[default]
    Vertical,
}

#[derive(Clone, PartialEq, Default, Copy)]
struct DragState {
    dragging: bool,
    offset: f32,
}

fn style_vsplitter(ss: &mut StyleBuilder) {
    ss.align_items(ui::AlignItems::Center)
        .justify_content(ui::JustifyContent::Center)
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Column)
        .gap(8)
        .width(9)
        .background_color(colors::U2);
}

// The decorative handle inside the splitter.
fn style_vsplitter_inner(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .width(3)
        // .pointer_events(PointerEvents::None)
        .height(ui::Val::Percent(20.));
}

/// Splitter bar which can be dragged
pub struct Splitter {
    /// The current split value.
    pub value: Signal<f32>,

    /// Whether the splitter bar runs horizontally or vertically.
    pub direction: SplitterDirection,

    /// Callback involved with the new split value.
    pub on_change: Option<Callback<f32>>,
}

impl Splitter {
    /// Create a new splitter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the current split value.
    pub fn value(mut self, value: impl IntoSignal<f32>) -> Self {
        self.value = value.into_signal();
        self
    }

    /// Set the direction of the splitter.
    pub fn direction(mut self, direction: SplitterDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Set the callback to be invoked when the split value changes.
    pub fn on_change(mut self, on_change: Callback<f32>) -> Self {
        self.on_change = Some(on_change);
        self
    }
}

impl Default for Splitter {
    fn default() -> Self {
        Self {
            value: Signal::Constant(0.),
            direction: SplitterDirection::Vertical,
            on_change: None,
        }
    }
}

impl ViewTemplate for Splitter {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let id = cx.create_entity();
        let hovering = cx.create_hover_signal(id);
        let drag_state = cx.create_mutable::<DragState>(DragState::default());
        let current_offset = self.value;
        Element::<NodeBundle>::for_entity(id)
            .named("Splitter")
            // .class_names(CLS_DRAG.if_true(cx.read_atom(drag_state).dragging))
            .style(style_vsplitter)
            .insert((
                On::<Pointer<DragStart>>::run(move |world: &mut World| {
                    // Save initial value to use as drag offset.
                    drag_state.set(
                        world,
                        DragState {
                            dragging: true,
                            offset: current_offset.get(world),
                        },
                    );
                }),
                On::<Pointer<DragEnd>>::run(move |world: &mut World| {
                    drag_state.set(
                        world,
                        DragState {
                            dragging: false,
                            offset: current_offset.get(world),
                        },
                    );
                }),
                On::<Pointer<Drag>>::run({
                    let on_change = self.on_change;
                    move |world: &mut World| {
                        let event = world
                            .get_resource::<ListenerInput<Pointer<Drag>>>()
                            .unwrap();
                        let ev = event.distance;
                        let ds = drag_state.get(world);
                        if let Some(on_change) = on_change {
                            if ds.dragging {
                                world.run_callback(on_change, ev.x + ds.offset);
                            }
                        }
                    }
                }),
                On::<Pointer<PointerCancel>>::run(move |world: &mut World| {
                    println!("Splitter Cancel");
                    drag_state.set(
                        world,
                        DragState {
                            dragging: false,
                            offset: current_offset.get(world),
                        },
                    );
                }),
            ))
            .children(
                Element::<NodeBundle>::new()
                    .style(style_vsplitter_inner)
                    .create_effect(move |cx, ent| {
                        // Color change on hover / drag
                        let ds = drag_state.get(cx);
                        let is_hovering = hovering.get(cx);
                        let color = match (ds.dragging, is_hovering) {
                            (true, _) => colors::U3.lighter(0.05),
                            (false, true) => colors::U3.lighter(0.02),
                            (false, false) => colors::U3,
                        };
                        let mut bg = cx.world_mut().get_mut::<BackgroundColor>(ent).unwrap();
                        bg.0 = color.into();
                    }),
            )
    }
}
