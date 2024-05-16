use crate::{size::Size, RoundedCorners};
use bevy::{
    a11y::{
        accesskit::{NodeBuilder, Role},
        AccessibilityNode,
    },
    prelude::*,
    ui,
};
use bevy_reactor_core::{Callback, Cx, IntoSignal, Signal, EffectTarget};
use bevy_reactor_style::{StyleBuilder, StyleBuilderLayout, StyleHandle, StyleTuple, WithStyles};
use bevy_reactor_view::{ChildArray, ChildViewTuple, Element, IntoView, ParentView, ViewTemplate};


use super::{Button, ButtonVariant};

fn style_tool_palette(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Grid)
        .gap(1)
        // .justify_content(ui::JustifyContent::Center)
        // .align_items(ui::AlignItems::Center)
        // .align_content(ui::AlignContent::Center);
        .grid_auto_rows(vec![ui::GridTrack::default()]);
}

#[derive(Clone, Debug, Default, Component)]
struct ToolPaletteContext {
    size: Size,
}

/// ToolPalette - a grid of tool buttons
#[derive(Default)]
pub struct ToolPalette {
    /// Button size.
    pub size: Size,

    /// The buttons to display.
    pub children: ChildArray,

    /// Additional styles to be applied to the palette.
    pub style: StyleHandle,

    /// Number of button columns
    pub columns: u16,
}

impl ToolPalette {
    /// Create a new tool palette.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the button size.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Set the child views for this element.
    pub fn children<V: ChildViewTuple>(mut self, children: V) -> Self {
        self.children = children.to_child_array();
        self
    }

    /// Set additional styles to be applied to the palette.
    pub fn style<S: StyleTuple + 'static>(mut self, style: S) -> Self {
        self.style = style.into_handle();
        self
    }

    /// Set the number of button columns.
    pub fn columns(mut self, columns: u16) -> Self {
        self.columns = columns;
        self
    }
}

impl ViewTemplate for ToolPalette {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let columns = self.columns;

        cx.insert(ToolPaletteContext { size: self.size });

        Element::<NodeBundle>::new()
            .named("ToolPalette")
            .style((
                style_tool_palette,
                move |ss: &mut StyleBuilder| {
                    ss.grid_template_columns(vec![ui::RepeatedGridTrack::auto(columns)]);
                },
                self.style.clone(),
            ))
            .insert(AccessibilityNode::from(NodeBuilder::new(Role::Group)))
            .children(self.children.clone())
    }
}

/// A button in a ToolPalette.
pub struct ToolButton {
    /// Color variant - default, primary or danger.
    pub(crate) variant: Signal<ButtonVariant>,

    /// Whether the button is disabled.
    pub(crate) disabled: Signal<bool>,

    /// The content to display inside the button.
    pub(crate) children: ChildArray,

    /// Callback called when clicked
    pub(crate) on_click: Option<Callback>,

    /// The tab index of the button (default 0).
    pub(crate) tab_index: i32,

    /// Which corners to render rounded.
    pub(crate) corners: RoundedCorners,

    /// If true, set focus to this button when it's added to the UI.
    pub(crate) autofocus: bool,
}

impl ToolButton {
    /// Create a new tool button.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the button color variant.
    pub fn variant(mut self, variant: impl IntoSignal<ButtonVariant>) -> Self {
        self.variant = variant.into_signal();
        self
    }

    /// Set the button disabled state.
    /// TODO: Come up with some kind of IntoSignal conversion for this.
    pub fn disabled(mut self, disabled: impl IntoSignal<bool>) -> Self {
        self.disabled = disabled.into_signal();
        self
    }

    /// Set the child views for this element.
    pub fn children<V: ChildViewTuple>(mut self, children: V) -> Self {
        self.children = children.to_child_array();
        self
    }

    /// Set callback when clicked
    pub fn on_click(mut self, callback: Callback) -> Self {
        self.on_click = Some(callback);
        self
    }

    /// Set the tab index of the button.
    pub fn tab_index(mut self, tab_index: i32) -> Self {
        self.tab_index = tab_index;
        self
    }

    /// Set which corners to render rounded.
    pub fn corners(mut self, corners: RoundedCorners) -> Self {
        self.corners = corners;
        self
    }

    /// Set whether to autofocus the button when it's added to the UI.
    pub fn autofocus(mut self, autofocus: bool) -> Self {
        self.autofocus = autofocus;
        self
    }
}

impl Default for ToolButton {
    fn default() -> Self {
        Self {
            variant: Default::default(),
            disabled: Default::default(),
            children: Default::default(),
            on_click: Default::default(),
            tab_index: 0,
            corners: RoundedCorners::None,
            autofocus: false,
        }
    }
}

impl ViewTemplate for ToolButton {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let context = cx.use_inherited_component::<ToolPaletteContext>().unwrap();
        let mut btn = Button::new()
            .size(context.size)
            .variant(self.variant)
            .disabled(self.disabled)
            .children(self.children.clone())
            // .on_click(self.on_click)
            .tab_index(self.tab_index)
            .autofocus(self.autofocus)
            .corners(self.corners);
        btn.on_click = self.on_click;
        btn
    }
}
