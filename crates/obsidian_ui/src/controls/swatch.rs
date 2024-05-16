use bevy::{color::Srgba, prelude::*, ui};
use bevy_mod_picking::prelude::*;
use bevy_reactor_core::{Callback, Cx, EffectTarget, IntoSignal, RunContextSetup, RunContextWrite, Signal};
use bevy_reactor_style::*;
use bevy_reactor_view::{Cond, Element, IntoView, ParentView, ViewTemplate};
// use bevy_tabindex::TabIndex;

use crate::materials::SwatchRectMaterial;

use crate::colors;

fn style_swatch(ss: &mut StyleBuilder) {
    ss.min_width(8)
        .min_height(8)
        .display(ui::Display::Flex)
        .color(colors::FOREGROUND)
        .padding(2);
}

fn style_selection(ss: &mut StyleBuilder) {
    ss.border(1)
        .border_color(colors::U1)
        .outline_color(colors::FOREGROUND)
        .outline_width(2)
        .outline_offset(0)
        .align_self(ui::AlignSelf::Stretch)
        .justify_self(ui::JustifySelf::Stretch)
        .flex_grow(1.);
}

/// Color swatch widget. This displays a solid color, and can also display a checkerboard
/// pattern behind the color if it has an alpha of less than 1.
#[derive(Default)]
pub struct Swatch {
    /// Color to display
    pub color: Signal<Srgba>,

    /// For swatch grids, whether this swatch is selected.
    pub selected: Signal<bool>,

    /// Additional styles to be applied to the widget.
    pub style: StyleHandle,

    /// Callback called when clicked
    pub on_click: Option<Callback<Srgba>>,
}

impl Swatch {
    /// Create a new swatch.
    pub fn new(color: impl IntoSignal<Srgba>) -> Self {
        Self::default().color(color.into_signal())
    }

    /// Set the color to display.
    pub fn color(mut self, color: impl Into<Signal<Srgba>>) -> Self {
        self.color = color.into();
        self
    }

    /// Set additional styles to be applied to the button.
    pub fn style<S: StyleTuple + 'static>(mut self, style: S) -> Self {
        self.style = style.into_handle();
        self
    }

    /// Set the callback called when clicked.
    pub fn on_click(mut self, on_click: Callback<Srgba>) -> Self {
        self.on_click = Some(on_click);
        self
    }

    /// Set whether the swatch should be rendered in a 'selected' state.
    pub fn selected(mut self, selected: impl Into<Signal<bool>>) -> Self {
        self.selected = selected.into();
        self
    }
}

impl ViewTemplate for Swatch {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let color = self.color;
        let selected = self.selected;

        let mut ui_materials = cx
            .world_mut()
            .get_resource_mut::<Assets<SwatchRectMaterial>>()
            .unwrap();
        let material = ui_materials.add(SwatchRectMaterial {
            color: LinearRgba::from(colors::U1).to_vec4(),
            border_radius: Vec4::splat(0.),
        });

        // Update material color
        cx.create_effect({
            let material = material.clone();
            move |cx| {
                let color = color.get(cx);
                let mut ui_materials = cx
                    .world_mut()
                    .get_resource_mut::<Assets<SwatchRectMaterial>>()
                    .unwrap();
                let material = ui_materials.get_mut(material.id()).unwrap();
                material.color = color.to_vec4();
            }
        });

        Element::<MaterialNodeBundle<SwatchRectMaterial>>::new()
            .named("Swatch")
            .style((style_swatch, self.style.clone()))
            .insert((material.clone(), {
                let on_click = self.on_click;
                On::<Pointer<Click>>::run(move |world: &mut World| {
                    let color = color.get(world);
                    if let Some(on_click) = on_click {
                        world.run_callback(on_click, color);
                    }
                })
            }))
            .children(Cond::new(
                move |cx| selected.get(cx),
                || Element::<NodeBundle>::new().style(style_selection),
                || (),
            ))
            .create_effect(move |cx, ent| {
                let radius = cx.use_component::<BorderRadius>(ent);
                if let Some(radius) = radius {
                    let radius = Vec4::from_array(resolve_border_radius(radius));
                    let mut ui_materials = cx
                        .world_mut()
                        .get_resource_mut::<Assets<SwatchRectMaterial>>()
                        .unwrap();
                    let material = ui_materials.get_mut(material.id()).unwrap();
                    material.border_radius = radius;
                }
            })
    }
}

// For now we only support pixel units.
fn resolve_border_radius(&values: &BorderRadius) -> [f32; 4] {
    [
        values.top_left,
        values.top_right,
        values.bottom_right,
        values.bottom_left,
    ]
    .map(|value| match value {
        Val::Px(px) => px,
        _ => 0.,
    })
}
