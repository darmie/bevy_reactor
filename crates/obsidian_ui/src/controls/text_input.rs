use crate::{
    focus::{AutoFocus, KeyCharEvent, KeyPressEvent, TabIndex},
    hooks::CreateFocusSignal,
};
use bevy::{
    a11y::{
        accesskit::{NodeBuilder, Role},
        AccessibilityNode, Focus,
    },
    color::Luminance,
    prelude::*,
    text::{BreakLineOn, TextLayoutInfo},
    ui,
};
use bevy_mod_picking::{events::PointerCancel, prelude::*};
use bevy_reactor_core::{Callback, CreateHoverSignal, Cx, RunContextRead, RunContextSetup, RunContextWrite, Signal, EffectTarget};
use bevy_reactor_style::*;
use bevy_reactor_view::{Cond, Element, For, IntoView, ParentView, ViewRef, ViewTemplate};


use crate::{colors, size::Size};

/// Text input properties
#[derive(Default)]
pub struct TextInputProps {
    /// Text input vertical size.
    pub size: Size,

    /// Whether the widget is disabled.
    pub disabled: Signal<bool>,

    /// The text string.
    pub value: Signal<String>,

    /// Additional styles to be applied to the button.
    pub style: StyleHandle,

    /// Callback called when the text changes.
    pub on_change: Option<Callback<String>>,

    /// The tab index of the text input (default 0).
    pub tab_index: i32,

    /// If true, set focus to this widget when it's added to the UI.
    pub autofocus: bool,

    /// Adornments to be placed before the input field.
    pub adornments_prefix: ViewRef,

    /// Adornments to be placed after the input field.
    pub adornments_suffix: ViewRef,
}

fn style_text_input(ss: &mut StyleBuilder) {
    ss.border(1)
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .justify_content(ui::JustifyContent::Stretch)
        .align_items(ui::AlignItems::Center)
        .align_content(ui::AlignContent::Center)
        .padding((4, 0))
        .border(0)
        .color(colors::FOREGROUND);
}

fn style_text_input_border(ss: &mut StyleBuilder) {
    ss.position(ui::PositionType::Absolute)
        .left(0)
        .right(0)
        .top(0)
        .bottom(0)
        .border_radius(5.0);
}

fn style_text_scroll(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        // .border(1)
        // .border_color(colors::CODE)
        .overflow(ui::OverflowAxis::Clip)
        .align_self(ui::AlignSelf::Stretch)
        .align_items(ui::AlignItems::Center)
        .flex_grow(1.)
        .flex_shrink(1.)
        .flex_basis(10)
        .min_width(0);
}

fn style_text_inner(ss: &mut StyleBuilder) {
    ss.position(ui::PositionType::Relative).left(0);
}

fn style_text_cursor(ss: &mut StyleBuilder) {
    ss.position(ui::PositionType::Absolute)
        .background_color(colors::TEXT_SELECT)
        .width(2)
        .height(16);
}

fn style_text_selection(ss: &mut StyleBuilder) {
    ss.position(ui::PositionType::Absolute)
        .background_color(colors::TEXT_SELECT);
}

/// Selection state for a text input.
#[derive(Default, Copy, Clone, PartialEq, Debug)]
struct Selection {
    /// The "cursor" is the position of the text input cursor. It is updated when the user
    /// drags to select text, or uses the arrow keys while shift is held.
    cursor: usize,

    /// The "anchor" is the position of the cursor when the user started the selection. It remains
    /// fixed while the user drags to select text.
    anchor: usize,
}

impl Selection {
    /// Create a new selection state.
    fn new(cursor: usize, anchor: usize) -> Self {
        Self { cursor, anchor }
    }

    /// Create a selection state with a single cursor position.
    fn single(cursor: usize) -> Self {
        Self::new(cursor, cursor)
    }

    /// Check if the selection is empty.
    fn is_empty(&self) -> bool {
        self.cursor == self.anchor
    }

    /// Get the start of the selection.
    fn start(&self) -> usize {
        self.cursor.min(self.anchor)
    }

    /// Get the end of the selection.
    fn end(&self) -> usize {
        self.cursor.max(self.anchor)
    }

    /// Get a range object representing the selection.
    fn range(&self) -> std::ops::Range<usize> {
        self.start()..self.end()
    }
}

/// Text input field.
pub struct TextInput(TextInputProps);

impl TextInput {
    /// Create a new text input control.
    pub fn new(props: TextInputProps) -> Self {
        Self(props)
    }
}

impl ViewTemplate for TextInput {
    #[allow(clippy::vec_init_then_push)]
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let id = cx.create_entity();
        let text_id = cx.create_entity();
        let hovering = cx.create_hover_signal(id);
        let focused = cx.create_focus_signal(id);
        let selection = cx.create_mutable::<Selection>(Selection::default());

        let disabled = self.0.disabled;

        let size = self.0.size;

        let server = cx.world_mut().get_resource::<AssetServer>().unwrap();
        let font = server.load("obsidian_ui://fonts/Open_Sans/static/OpenSans-Medium.ttf");

        let value = self.0.value.clone();

        // Derived signal that computes the selection rectangles.
        let selection_rects = cx.create_derived(move |cx| {
            let selection = selection.get(cx);
            let mut rects = Vec::<Rect>::new();
            if !focused.get(cx) {
                return rects;
            }
            if let Some(text_layout) = cx.use_component::<TextLayoutInfo>(text_id) {
                if !selection.is_empty() {
                    let start = selection.start();
                    let end = selection.end();
                    let start_glyph = &text_layout.glyphs[start];
                    let end_glyph = &text_layout.glyphs[end - 1];
                    rects.push(Rect::new(
                        start_glyph.position.x * 0.5,
                        (start_glyph.position.y - start_glyph.size.y)
                            .min(end_glyph.position.y - end_glyph.size.y),
                        end_glyph.position.x * 0.5 + end_glyph.size.x,
                        start_glyph.position.y.max(end_glyph.position.y),
                    ));
                }
            }
            rects
        });

        Element::<NodeBundle>::for_entity(id)
            .named("text_input")
            .style((
                style_text_input,
                move |ss: &mut StyleBuilder| {
                    ss.min_height(size.height());
                },
                self.0.style.clone(),
            ))
            .insert((
                TabIndex(self.0.tab_index),
                AccessibilityNode::from(NodeBuilder::new(Role::TextInput)),
                {
                    // let on_click = self.0.on_click;
                    On::<Pointer<Click>>::run(move |world: &mut World| {
                        if !disabled.get(world) {
                            //     if let Some(on_click) = on_click {
                            //     world.run_callback(on_click, ());
                            // }
                        }
                    })
                },
                On::<Pointer<DragStart>>::run(move |world: &mut World| {
                    let mut focus = world.get_resource_mut::<Focus>().unwrap();
                    focus.0 = Some(id);
                    if !disabled.get(world) {
                        // pressed.set(world, true);
                    }
                }),
                On::<Pointer<DragEnd>>::run(move |world: &mut World| {
                    if !disabled.get(world) {
                        // pressed.set(world, false);
                    }
                }),
                On::<Pointer<PointerCancel>>::run(move |world: &mut World| {
                    println!("PointerCancel");
                    if !disabled.get(world) {
                        // pressed.set(world, false);
                    }
                }),
                On::<KeyCharEvent>::run({
                    let on_change = self.0.on_change;
                    let value = value.clone();
                    move |world: &mut World| {
                        if !disabled.get(world) {
                            let mut text_value = value.get_clone(world);
                            let sel = selection.get(world);
                            let mut event = world
                                .get_resource_mut::<ListenerInput<KeyCharEvent>>()
                                .unwrap();
                            if !event.key.is_control() {
                                text_value.replace_range(sel.range(), &event.key.to_string());
                                let new_cursor_pos = sel.start() + 1;
                                event.stop_propagation();
                                if let Some(on_change) = on_change {
                                    world.run_callback(on_change, text_value);
                                    selection.set(world, Selection::single(new_cursor_pos));
                                }
                            }
                        }
                    }
                }),
                On::<KeyPressEvent>::run({
                    let on_change = self.0.on_change;
                    let value = value.clone();
                    move |world: &mut World| {
                        if !disabled.get(world) {
                            let text_len = value.map(world, |v| v.len());
                            let sel = selection.get(world);
                            let event = world
                                .get_resource_mut::<ListenerInput<KeyPressEvent>>()
                                .unwrap();
                            let mut handled = false;
                            match event.key_code {
                                KeyCode::ArrowLeft => {
                                    if sel.cursor > 0 {
                                        if event.shift {
                                            selection.set(
                                                world,
                                                Selection::new(sel.cursor - 1, sel.anchor),
                                            );
                                        } else {
                                            selection.set(world, Selection::single(sel.cursor - 1));
                                        }
                                        handled = true;
                                    }
                                }

                                KeyCode::ArrowRight => {
                                    if sel.cursor < text_len {
                                        if event.shift {
                                            selection.set(
                                                world,
                                                Selection::new(sel.cursor + 1, sel.anchor),
                                            );
                                        } else {
                                            selection.set(world, Selection::single(sel.cursor + 1));
                                        }
                                        handled = true;
                                    }
                                }

                                KeyCode::ArrowUp => {
                                    // TODO: For multiline text inputs, move the cursor up a line.
                                    handled = true;
                                }

                                KeyCode::ArrowDown => {
                                    // TODO: For multiline text inputs, move the cursor down a line.
                                    handled = true;
                                }

                                KeyCode::Home => {
                                    if sel.cursor > 0 {
                                        selection.set(world, Selection::single(0));
                                        handled = true;
                                    }
                                }

                                KeyCode::End => {
                                    if sel.cursor < text_len {
                                        selection.set(world, Selection::single(text_len));
                                        handled = true;
                                    }
                                }

                                KeyCode::Backspace => {
                                    let mut new_text = value.get_clone(world);
                                    if sel.is_empty() {
                                        if sel.cursor > 0 {
                                            new_text.remove(sel.cursor - 1);
                                            if let Some(on_change) = on_change {
                                                world.run_callback(on_change, new_text);
                                            }
                                            selection.set(world, Selection::single(sel.cursor - 1));
                                        }
                                    } else {
                                        new_text.replace_range(sel.range(), "");
                                        if let Some(on_change) = on_change {
                                            world.run_callback(on_change, new_text);
                                        }
                                        selection.set(world, Selection::single(sel.start()));
                                    }
                                    handled = true;
                                }

                                KeyCode::Delete => {
                                    let mut new_text = value.get_clone(world);
                                    if sel.is_empty() {
                                        if sel.cursor < new_text.len() {
                                            new_text.remove(sel.cursor);
                                            if let Some(on_change) = on_change {
                                                world.run_callback(on_change, new_text);
                                            }
                                            selection.set(world, Selection::single(sel.cursor));
                                        }
                                    } else {
                                        let mut new_text = value.get_clone(world);
                                        new_text.replace_range(sel.range(), "");
                                        if let Some(on_change) = on_change {
                                            world.run_callback(on_change, new_text);
                                        }
                                        selection.set(world, Selection::single(sel.start()));
                                    }
                                    handled = true;
                                }
                                _ => {}
                            }

                            if handled {
                                let mut event = world
                                    .get_resource_mut::<ListenerInput<KeyPressEvent>>()
                                    .unwrap();
                                event.stop_propagation();
                            }
                        }
                    }
                }),
            ))
            .insert_if(self.0.autofocus, AutoFocus)
            .children((
                // Background
                Element::<NodeBundle>::new()
                    .style(style_text_input_border)
                    .create_effect(move |cx, ent| {
                        let is_hovering = hovering.get(cx);
                        let is_focused = focused.get(cx);
                        let color = match (is_focused, is_hovering) {
                            (true, _) => colors::U1.lighter(0.05),
                            (false, true) => colors::U1.lighter(0.01),
                            (false, false) => colors::U1,
                        };
                        let mut bg = cx.world_mut().get_mut::<BackgroundColor>(ent).unwrap();
                        bg.0 = color.into();
                    })
                    .create_effect(move |cx, entt| {
                        let is_focused = focused.get(cx);
                        let mut entt = cx.world_mut().entity_mut(entt);
                        // TODO: Don't do this as an outline, do it as an inset border.
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
                    }),
                // Prefix adornments
                self.0.adornments_prefix.clone(),
                // Scrolling content
                Element::<NodeBundle>::new()
                    .style(style_text_scroll)
                    .children(
                        Element::<NodeBundle>::new()
                            .style(style_text_inner)
                            .children((
                                // Selection rects
                                For::index(
                                    move |cx| selection_rects.get_clone(cx).into_iter(),
                                    move |rect, _| {
                                        Element::<NodeBundle>::new().style((
                                            style_text_selection,
                                            {
                                                let rect = *rect;
                                                move |ss: &mut StyleBuilder| {
                                                    ss.left(ui::Val::Px(rect.min.x))
                                                        .top(ui::Val::Px(rect.min.y))
                                                        .width(ui::Val::Px(rect.width()))
                                                        .height(ui::Val::Px(rect.height()));
                                                }
                                            },
                                        ))
                                    },
                                ),
                                // Caret
                                Cond::new(
                                    move |cx| {
                                        selection.get(cx).is_empty()
                                            && focused.get(cx)
                                            && !disabled.get(cx)
                                    },
                                    // React to changes in glyph layout.
                                    move || {
                                        Element::<NodeBundle>::new()
                                            .style(style_text_cursor)
                                            .create_effect(move |cx, el| {
                                                let index = selection.get(cx).cursor;
                                                let text_layout = cx
                                                    .use_component::<TextLayoutInfo>(text_id)
                                                    .unwrap();
                                                let mut pos: Vec2 = Vec2::default();
                                                let height: f32;
                                                if index >= text_layout.glyphs.len() {
                                                    let glyph = text_layout.glyphs.last().unwrap();
                                                    pos.x = glyph.position.x + glyph.size.x;
                                                    pos.y = glyph.position.y;
                                                    height = glyph.size.y;
                                                } else {
                                                    let glyph = &text_layout.glyphs[index];
                                                    pos.x = glyph.position.x;
                                                    pos.y = glyph.position.y;
                                                    height = glyph.size.y;
                                                }
                                                let mut entt = cx.world_mut().entity_mut(el);
                                                let mut style = entt.get_mut::<Style>().unwrap();
                                                style.left = ui::Val::Px(pos.x * 0.5);
                                                style.top = ui::Val::Px(pos.y - height);
                                                style.height = ui::Val::Px(height);
                                            })
                                    },
                                    || (),
                                ),
                                // Text
                                Element::<TextBundle>::for_entity(text_id).create_effect(
                                    move |cx, elem| {
                                        let sections = value.map(cx, |s| {
                                            let mut sections: Vec<TextSection> = Vec::new();
                                            sections.push(TextSection {
                                                value: s[..].to_string(),
                                                style: TextStyle {
                                                    font: font.clone(),
                                                    font_size: 16.0,
                                                    color: colors::FOREGROUND.into(),
                                                },
                                            });
                                            sections
                                        });
                                        let mut entt = cx.world_mut().entity_mut(elem);
                                        if let Some(mut text) = entt.get_mut::<Text>() {
                                            text.linebreak_behavior = BreakLineOn::NoWrap;
                                            text.sections = sections;
                                        }
                                    },
                                ),
                            )),
                    ),
                // Suffix adornments
                self.0.adornments_suffix.clone(),
            ))
    }
}
