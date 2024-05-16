use bevy::{prelude::*, ui};
use bevy_reactor_style::*;
use bevy_reactor_view::*;
use bevy_reactor_core::*;


use crate::colors;

use super::ScrollView;

fn style_listview(ss: &mut StyleBuilder) {
    ss.background_color(colors::U1)
        .border_radius(5.0)
        .padding(3);
}

fn style_listview_inner(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Column)
        .align_items(ui::AlignItems::Stretch);
}

/// A scrollable list of items.
#[derive(Clone, Default)]
pub struct ListView {
    /// Additional styles to be applied to the list view.
    pub style: StyleHandle,

    /// The content of the dialog header.
    pub children: ChildArray,
}

impl ListView {
    /// Create a new list view.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set additional styles to be applied to the list view.
    pub fn style<S: StyleTuple + 'static>(mut self, style: S) -> Self {
        self.style = style.into_handle();
        self
    }

    /// Set the child views for this element.
    pub fn children<V: ChildViewTuple>(mut self, children: V) -> Self {
        self.children = children.to_child_array();
        self
    }
}

impl ViewTemplate for ListView {
    fn create(&self, _cx: &mut Cx) -> impl IntoView {
        ScrollView::new()
            .children(
                Element::<NodeBundle>::new()
                    .named("ListView")
                    .style(style_listview_inner)
                    .children(self.children.clone()),
            )
            .style((style_listview, self.style.clone()))
            .scroll_enable_y(true)
    }
}
