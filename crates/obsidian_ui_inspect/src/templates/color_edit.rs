use bevy::{
    color::{Alpha, Hsla, Hue, Srgba},
    ecs::system::Resource,
    math::UVec2,
    ui::{self, node_bundles::NodeBundle},
};
use bevy_reactor_core::*;
use bevy_reactor_style::*;
use bevy_reactor_view::*;
use obsidian_ui::{
    controls::{Button, ButtonVariant, ColorGradient, GradientSlider, Swatch, SwatchGrid},
    RoundedCorners,
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ColorMode {
    #[default]
    Rgb,
    Hsl,
    Recent,
}

/// State for the color edit control. The state stores all color spaces simultaneously to avoid
/// precision loss when converting between them.
#[derive(Default, Clone, Copy, PartialEq)]
pub struct ColorEditState {
    pub mode: ColorMode,
    pub rgb: Srgba,
    pub hsl: Hsla,
}

const MAX_RECENT: usize = 32;

fn style_recent_colors(ss: &mut StyleBuilder) {
    ss.align_self(ui::AlignSelf::Stretch).height(76);
}

/// Recent colors for the color edit control.
#[derive(Resource, Default, Clone)]
pub struct RecentColors(pub Vec<Srgba>);

impl RecentColors {
    pub fn add(&mut self, color: Srgba) {
        // Move color to front of list, removing duplicates.
        if let Some(index) = self.0.iter().position(|c| *c == color) {
            self.0.remove(index);
        }
        // Add color to front of list and trim.
        self.0.insert(0, color);
        if self.0.len() > MAX_RECENT {
            self.0.pop();
        }
    }
}

impl ColorEditState {
    pub fn set_mode(self, mode: ColorMode) -> Self {
        let mut result = self;
        result.mode = mode;
        result
    }

    pub fn set_rgb(self, rgb: Srgba) -> Self {
        let mut result = self;
        result.rgb = rgb;
        result.hsl = rgb.into();
        // Preserve hue if saturation is near zero, or lightness is close to full white or black.
        if self.hsl.saturation < 0.00001
            || self.hsl.lightness < 0.00001
            || self.hsl.lightness > 9.99999
        {
            result.hsl.hue = self.hsl.hue;
        }
        result
    }

    pub fn set_red(self, value: f32) -> Self {
        self.set_rgb(self.rgb.with_red(value))
    }

    pub fn set_green(self, value: f32) -> Self {
        self.set_rgb(self.rgb.with_green(value))
    }

    pub fn set_blue(self, value: f32) -> Self {
        self.set_rgb(self.rgb.with_blue(value))
    }

    pub fn set_hsl(self, hsl: Hsla) -> Self {
        let mut result = self;
        result.hsl = hsl;
        result.rgb = hsl.into();
        result
    }

    pub fn set_hue(self, value: f32) -> Self {
        self.set_hsl(self.hsl.with_hue(value))
    }

    pub fn set_saturation(self, value: f32) -> Self {
        self.set_hsl(self.hsl.with_saturation(value))
    }

    pub fn set_lightness(self, value: f32) -> Self {
        self.set_hsl(self.hsl.with_lightness(value))
    }

    pub fn set_alpha(self, alpha: f32) -> Self {
        let mut result = self;
        result.rgb.alpha = alpha;
        result.hsl.alpha = alpha;
        result
    }
}

fn style_grid(sb: &mut StyleBuilder) {
    sb.display(ui::Display::Flex)
        .align_items(ui::AlignItems::Stretch)
        .flex_direction(ui::FlexDirection::Column)
        .min_width(240)
        .margin((8, 4))
        .gap(4);
}

fn style_top_row(sb: &mut StyleBuilder) {
    sb.display(ui::Display::Flex)
        .align_items(ui::AlignItems::Stretch)
        .flex_direction(ui::FlexDirection::Row)
        .gap(4)
        .margin_bottom(4);
}

fn style_mode_selector(sb: &mut StyleBuilder) {
    sb.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .justify_content(ui::JustifyContent::Center)
        .align_items(ui::AlignItems::Center)
        .gap(1);
}

fn style_sliders(sb: &mut StyleBuilder) {
    sb.display(ui::Display::Grid)
        .grid_template_columns(vec![
            ui::RepeatedGridTrack::fr(1, 1.),
            ui::RepeatedGridTrack::px(1, 32.),
        ])
        .grid_auto_flow(ui::GridAutoFlow::Row)
        .align_items(ui::AlignItems::Center)
        .row_gap(4)
        .column_gap(4);
}

fn style_slider(ss: &mut StyleBuilder) {
    ss.align_self(ui::AlignSelf::Stretch);
}

fn style_numeric_input(ss: &mut StyleBuilder) {
    ss.align_self(ui::AlignSelf::Center)
        .justify_self(ui::JustifySelf::End);
}

fn style_swatch(ss: &mut StyleBuilder) {
    ss.align_self(ui::AlignSelf::Stretch)
        .flex_grow(1.)
        .border_radius(5.);
}

pub struct ColorEdit {
    state: Signal<ColorEditState>,
    on_change: Callback<ColorEditState>,
}

impl ColorEdit {
    /// Create a new color edit control.
    pub fn new(
        state: impl IntoSignal<ColorEditState>,
        on_change: Callback<ColorEditState>,
    ) -> Self {
        Self {
            state: state.into_signal(),
            on_change,
        }
    }
}

impl ViewTemplate for ColorEdit {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let state = self.state;
        let mode = cx.create_memo(move |cx| state.map(cx, |st| st.mode));
        let on_change = self.on_change;

        cx.on_cleanup(move |world| {
            // Add color to recent colors.
            let color = state.get(world).rgb;
            let mut recent_colors = world.get_resource_mut::<RecentColors>().unwrap();
            recent_colors.add(color);
        });

        Element::<NodeBundle>::new().style(style_grid).children((
            Element::<NodeBundle>::new().style(style_top_row).children((
                Swatch::new(cx.create_memo(move |cx| state.get(cx).rgb)).style(style_swatch),
                Element::<NodeBundle>::new()
                    .style(style_mode_selector)
                    .children((
                        Button::new()
                            .children("RGB")
                            .corners(RoundedCorners::Left)
                            .variant(cx.create_derived(move |cx| {
                                if mode.get(cx) == ColorMode::Rgb {
                                    ButtonVariant::Selected
                                } else {
                                    ButtonVariant::Default
                                }
                            }))
                            .on_click(cx.create_callback(move |cx: &mut Cx, ()| {
                                cx.run_callback(on_change, state.get(cx).set_mode(ColorMode::Rgb));
                            })),
                        Button::new()
                            .children("HSL")
                            .corners(RoundedCorners::None)
                            .variant(cx.create_derived(move |cx| {
                                if mode.get(cx) == ColorMode::Hsl {
                                    ButtonVariant::Selected
                                } else {
                                    ButtonVariant::Default
                                }
                            }))
                            .on_click(cx.create_callback(move |cx: &mut Cx, ()| {
                                cx.run_callback(on_change, state.get(cx).set_mode(ColorMode::Hsl));
                            })),
                        Button::new()
                            .children("Recent")
                            .corners(RoundedCorners::Right)
                            .variant(cx.create_derived(move |cx| {
                                if mode.get(cx) == ColorMode::Recent {
                                    ButtonVariant::Selected
                                } else {
                                    ButtonVariant::Default
                                }
                            }))
                            .on_click(cx.create_callback(move |cx: &mut Cx, ()| {
                                cx.run_callback(
                                    on_change,
                                    state.get(cx).set_mode(ColorMode::Recent),
                                );
                            })),
                    )),
            )),
            DynamicKeyed::new(
                move |cx| mode.get(cx),
                move |mode| match mode {
                    ColorMode::Rgb => RgbSliders { state, on_change }.into_view(),
                    ColorMode::Hsl => HslSliders { state, on_change }.into_view(),
                    ColorMode::Recent => RecentColorsGrid { state, on_change }.into_view(),
                },
            ),
        ))
    }
}

struct RgbSliders {
    state: Signal<ColorEditState>,
    on_change: Callback<ColorEditState>,
}

impl ViewTemplate for RgbSliders {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let state = self.state;
        let rgb = cx.create_memo(move |cx| state.map(cx, |st| st.rgb));
        let on_change = self.on_change;

        Element::<NodeBundle>::new().style(style_sliders).children((
            GradientSlider::new()
                .gradient(cx.create_derived(move |cx| {
                    let rgb = rgb.get(cx);
                    ColorGradient::new(&[
                        Srgba::new(0.0, rgb.green, rgb.blue, 1.0),
                        Srgba::new(1.0, rgb.green, rgb.blue, 1.0),
                    ])
                }))
                .min(0.)
                .max(255.)
                .value(cx.create_derived(move |cx| rgb.get(cx).red * 255.0))
                .style(style_slider)
                .precision(1)
                .on_change(cx.create_callback(move |cx, value| {
                    cx.run_callback(on_change, state.get(cx).set_red(value / 255.0));
                })),
            Element::<NodeBundle>::new()
                .style(style_numeric_input)
                .children(text_computed(move |cx| {
                    format!("{:.0}", rgb.get(cx).red * 255.0)
                })),
            GradientSlider::new()
                .gradient(cx.create_derived(move |cx| {
                    let rgb = rgb.get(cx);
                    ColorGradient::new(&[
                        Srgba::new(rgb.red, 0.0, rgb.blue, 1.0),
                        Srgba::new(rgb.red, 1.0, rgb.blue, 1.0),
                    ])
                }))
                .min(0.)
                .max(255.)
                .value(cx.create_derived(move |cx| rgb.get(cx).green * 255.0))
                .style(style_slider)
                .precision(1)
                .on_change(cx.create_callback(move |cx, value| {
                    cx.run_callback(on_change, state.get(cx).set_green(value / 255.0));
                })),
            Element::<NodeBundle>::new()
                .style(style_numeric_input)
                .children(text_computed(move |cx| {
                    format!("{:.0}", rgb.get(cx).green * 255.0)
                })),
            GradientSlider::new()
                .gradient(cx.create_derived(move |cx| {
                    let rgb = rgb.get(cx);
                    ColorGradient::new(&[
                        Srgba::new(rgb.red, rgb.green, 0.0, 1.0),
                        Srgba::new(rgb.red, rgb.green, 1.0, 1.0),
                    ])
                }))
                .min(0.)
                .max(255.)
                .value(cx.create_derived(move |cx| rgb.get(cx).blue * 255.0))
                .style(style_slider)
                .precision(1)
                .on_change(cx.create_callback(move |cx, value| {
                    cx.run_callback(on_change, state.get(cx).set_blue(value / 255.0));
                })),
            Element::<NodeBundle>::new()
                .style(style_numeric_input)
                .children(text_computed(move |cx| {
                    format!("{:.0}", rgb.get(cx).blue * 255.0)
                })),
            AlphaSlider { state, on_change },
        ))
    }
}

struct HslSliders {
    state: Signal<ColorEditState>,
    on_change: Callback<ColorEditState>,
}

impl ViewTemplate for HslSliders {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let state = self.state;
        let hsl = cx.create_memo(move |cx| state.map(cx, |st| st.hsl));
        let on_change = self.on_change;

        Element::<NodeBundle>::new().style(style_sliders).children((
            GradientSlider::new()
                .gradient(ColorGradient::new(&[
                    Srgba::from(Hsla::new(0.0, 1.0, 0.5, 1.0)),
                    Srgba::from(Hsla::new(60.0, 1.0, 0.5, 1.0)),
                    Srgba::from(Hsla::new(120.0, 1.0, 0.5, 1.0)),
                    Srgba::from(Hsla::new(180.0, 1.0, 0.5, 1.0)),
                    Srgba::from(Hsla::new(240.0, 1.0, 0.5, 1.0)),
                    Srgba::from(Hsla::new(300.0, 1.0, 0.5, 1.0)),
                    Srgba::from(Hsla::new(360.0, 1.0, 0.5, 1.0)),
                ]))
                .min(0.)
                .max(360.)
                .value(cx.create_derived(move |cx| hsl.get(cx).hue))
                .style(style_slider)
                .precision(1)
                .on_change(cx.create_callback(move |cx, value| {
                    cx.run_callback(on_change, state.get(cx).set_hue(value));
                })),
            Element::<NodeBundle>::new()
                .style(style_numeric_input)
                .children(text_computed(move |cx| format!("{:.0}", hsl.get(cx).hue))),
            GradientSlider::new()
                .gradient(cx.create_derived(move |cx| {
                    let hsl = hsl.get(cx);
                    ColorGradient::new(&[
                        Srgba::from(Hsla::new(hsl.hue, 0.0, hsl.lightness, 1.0)),
                        Srgba::from(Hsla::new(hsl.hue, 0.5, hsl.lightness, 1.0)),
                        Srgba::from(Hsla::new(hsl.hue, 1.0, hsl.lightness, 1.0)),
                    ])
                }))
                .min(0.)
                .max(100.)
                .value(cx.create_derived(move |cx| hsl.get(cx).saturation * 100.0))
                .style(style_slider)
                .precision(1)
                .on_change(cx.create_callback(move |cx, value| {
                    cx.run_callback(on_change, state.get(cx).set_saturation(value / 100.0));
                })),
            Element::<NodeBundle>::new()
                .style(style_numeric_input)
                .children(text_computed(move |cx| {
                    format!("{:.0}", hsl.get(cx).saturation * 100.0)
                })),
            GradientSlider::new()
                .gradient(cx.create_derived(move |cx| {
                    let hsl = hsl.get(cx);
                    ColorGradient::new(&[
                        Srgba::from(Hsla::new(hsl.hue, hsl.saturation, 0.0, 1.0)),
                        Srgba::from(Hsla::new(hsl.hue, hsl.saturation, 0.5, 1.0)),
                        Srgba::from(Hsla::new(hsl.hue, hsl.saturation, 1.0, 1.0)),
                    ])
                }))
                .min(0.)
                .max(100.)
                .value(cx.create_derived(move |cx| hsl.get(cx).lightness * 100.0))
                .style(style_slider)
                .precision(1)
                .on_change(cx.create_callback(move |cx, value| {
                    cx.run_callback(on_change, state.get(cx).set_lightness(value / 100.0));
                })),
            Element::<NodeBundle>::new()
                .style(style_numeric_input)
                .children(text_computed(move |cx| {
                    format!("{:.0}", hsl.get(cx).lightness * 100.0)
                })),
            AlphaSlider { state, on_change },
        ))
    }
}

struct AlphaSlider {
    state: Signal<ColorEditState>,
    on_change: Callback<ColorEditState>,
}

impl ViewTemplate for AlphaSlider {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let state = self.state;
        let rgb = cx.create_memo(move |cx| state.map(cx, |st| st.rgb));
        let on_change = self.on_change;

        Fragment::new((
            GradientSlider::new()
                .gradient(cx.create_derived(move |cx| {
                    let base_color = rgb.get(cx);
                    ColorGradient::new(&[base_color.with_alpha(0.), base_color.with_alpha(1.)])
                }))
                .min(0.)
                .max(255.)
                .value(cx.create_derived(move |cx| rgb.get(cx).alpha * 255.0))
                .style(style_slider)
                .precision(1)
                .on_change(cx.create_callback(move |cx, value| {
                    cx.run_callback(on_change, state.get(cx).set_alpha(value / 255.0));
                })),
            Element::<NodeBundle>::new()
                .style(style_numeric_input)
                .children(text_computed(move |cx| {
                    format!("{:.0}", rgb.get(cx).alpha * 255.0)
                })),
        ))
    }
}

struct RecentColorsGrid {
    state: Signal<ColorEditState>,
    on_change: Callback<ColorEditState>,
}

impl ViewTemplate for RecentColorsGrid {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let state = self.state;
        let rgb = cx.create_memo(move |cx| state.map(cx, |st| st.rgb));
        let on_change = self.on_change;
        let recent_colors = cx.create_derived(|cx| cx.use_resource::<RecentColors>().0.clone());

        SwatchGrid::new(recent_colors)
            .style(style_recent_colors)
            .grid_size(UVec2::new(12, 4))
            .selected(rgb)
            .on_change(cx.create_callback(move |cx, color| {
                cx.run_callback(on_change, state.get(cx).set_rgb(color));
            }))
    }
}
