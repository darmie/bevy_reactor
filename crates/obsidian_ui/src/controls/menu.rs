use crate::{
    colors,
    floating::{FloatAlign, FloatPosition, FloatSide, Floating},
    focus::{AutoFocus, KeyPressEvent, NavAction, TabGroup, TabIndex, TabNavigation},
    hooks::{BistableTransitionState, CreateBistableTransition, CreateFocusSignal},
    size::Size,
    typography, RoundedCorners,
};
use bevy::{
    a11y::{
        accesskit::{HasPopup, NodeBuilder, Role},
        AccessibilityNode, Focus,
    },
    ecs::system::SystemState,
    prelude::*,
    ui,
};
use bevy_mod_picking::{events::PointerCancel, prelude::*};
use bevy_reactor_core::{Callback, CreateHoverSignal, Cx, EffectTarget, IntoSignal, RunContextSetup, RunContextWrite, Signal};
use bevy_reactor_style::*;
use bevy_reactor_view::{ChildArray, ChildView, ChildViewTuple, Cond, Element, IntoView, ParentView, Portal, ViewTemplate};

use super::{style_button, style_button_bg, ButtonVariant, Icon, Spacer};

/// View context component which stores the anchor element id for a menu.
#[derive(Component)]
struct MenuAnchor(Entity);

#[derive(Clone, Event, EntityEvent)]
#[can_bubble]
pub(crate) struct MenuCloseEvent {
    /// The target of the event
    #[target]
    pub target: Entity,
}

// Dialog background overlay
fn style_menu_barrier(ss: &mut StyleBuilder) {
    ss.position(PositionType::Absolute)
        .display(ui::Display::Flex)
        .justify_content(ui::JustifyContent::Center)
        .align_items(ui::AlignItems::Center)
        .left(0)
        .top(0)
        .right(0)
        .bottom(0)
        .z_index(100)
        .background_color(colors::U2.with_alpha(0.0));
}

/// A widget that displays a drop-down menu when clicked.
#[derive(Default)]
pub struct MenuButton {
    /// Id of the anchor element for the menu.
    pub anchor: Option<Entity>,

    /// Color variant - default, primary or danger.
    pub variant: Signal<ButtonVariant>,

    /// Button size.
    pub size: Size,

    /// Whether the button is disabled.
    pub disabled: Signal<bool>,

    /// Which corners to render rounded.
    pub corners: RoundedCorners,

    /// If true, set focus to this button when it's added to the UI.
    pub autofocus: bool,

    /// If true, render the button in a 'minimal' style with no background and reduced padding.
    pub minimal: bool,

    /// The content to display inside the button.
    pub children: ChildArray,

    /// Additional styles to be applied to the button.
    pub style: StyleHandle,

    /// The popup to display when the button is clicked.
    pub popup: ChildArray,

    /// If true, don't display the caret icon.
    pub no_caret: bool,

    /// The tab index of the button (default 0).
    pub tab_index: i32,
}

impl MenuButton {
    /// Create a new menu button.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the button color variant.
    pub fn variant(mut self, variant: impl IntoSignal<ButtonVariant>) -> Self {
        self.variant = variant.into_signal();
        self
    }

    /// Set the button size.
    pub fn size(mut self, size: Size) -> Self {
        self.size = size;
        self
    }

    /// Set the button disabled state.
    pub fn disabled(mut self, disabled: impl IntoSignal<bool>) -> Self {
        self.disabled = disabled.into_signal();
        self
    }

    /// Set the button corners.
    pub fn corners(mut self, corners: RoundedCorners) -> Self {
        self.corners = corners;
        self
    }

    /// Set the button autofocus state.
    pub fn autofocus(mut self, autofocus: bool) -> Self {
        self.autofocus = autofocus;
        self
    }

    /// Set the button minimal state.
    pub fn minimal(mut self, minimal: bool) -> Self {
        self.minimal = minimal;
        self
    }

    /// Set the button children.
    pub fn children<V: ChildViewTuple>(mut self, children: V) -> Self {
        self.children = children.to_child_array();
        self
    }

    /// Set the button style.
    pub fn style(mut self, style: StyleHandle) -> Self {
        self.style = style;
        self
    }

    /// Control whether to hide the drop-down caret icon.
    pub fn no_caret(mut self, no_caret: bool) -> Self {
        self.no_caret = no_caret;
        self
    }

    /// Set the button popup.
    pub fn popup<V: ChildViewTuple>(mut self, popup: V) -> Self {
        self.popup = popup.to_child_array();
        self
    }

    /// Set the button tab index.
    pub fn tab_index(mut self, tab_index: i32) -> Self {
        self.tab_index = tab_index;
        self
    }
}

impl ViewTemplate for MenuButton {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let id_anchor = self.anchor.unwrap_or_else(|| cx.create_entity());
        let variant = self.variant;
        let open = cx.create_mutable::<bool>(false);
        let hovering = cx.create_hover_signal(id_anchor);
        let focused = cx.create_focus_visible_signal(id_anchor);

        let disabled = self.disabled;
        let corners = self.corners;
        let minimal = self.minimal;

        let size = self.size;
        let popup = self.popup.clone();

        cx.insert(MenuAnchor(id_anchor));
        cx.insert(On::<MenuCloseEvent>::run(move |world: &mut World| {
            let mut event = world
                .get_resource_mut::<ListenerInput<MenuCloseEvent>>()
                .unwrap();
            event.stop_propagation();
            open.set(world, false);
        }));

        Element::<NodeBundle>::for_entity(id_anchor)
            .named("MenuButton")
            .style((
                typography::text_default,
                style_button,
                move |ss: &mut StyleBuilder| {
                    ss.min_height(size.height()).font_size(size.font_size());
                    if minimal {
                        ss.padding(0);
                    }
                },
                self.style.clone(),
            ))
            .insert((
                TabIndex(self.tab_index),
                On::<Pointer<Click>>::run(move |world: &mut World| {
                    let mut focus = world.get_resource_mut::<Focus>().unwrap();
                    focus.0 = Some(id_anchor);
                    if !disabled.get(world) {
                        let mut event = world
                            .get_resource_mut::<ListenerInput<Pointer<Click>>>()
                            .unwrap();
                        event.stop_propagation();
                        open.update(world, |mut state| {
                            *state = !*state;
                        });
                    }
                }),
                On::<KeyPressEvent>::run({
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
                                open.update(world, |mut state| {
                                    *state = !*state;
                                });
                            }
                        }
                    }
                }),
            ))
            .insert_computed(move |cx| {
                AccessibilityNode::from({
                    let mut builder = NodeBuilder::new(Role::Button);
                    builder.set_has_popup(HasPopup::Menu);
                    builder.set_expanded(open.get(cx));
                    builder
                })
            })
            .insert_if(self.autofocus, AutoFocus)
            .children((
                Element::<NodeBundle>::new()
                    .named("MenuButton::Background")
                    .style(style_button_bg)
                    .insert(corners.to_border_radius(self.size.border_radius()))
                    .create_effect(move |cx, ent| {
                        let is_pressed = open.get(cx);
                        let is_hovering = hovering.get(cx);
                        let base_color = match variant.get(cx) {
                            ButtonVariant::Default => colors::U3,
                            ButtonVariant::Primary => colors::PRIMARY,
                            ButtonVariant::Danger => colors::DESTRUCTIVE,
                            ButtonVariant::Selected => colors::U4,
                        };
                        let color = match (is_pressed, is_hovering) {
                            (true, _) => base_color.lighter(0.05),
                            (false, true) => base_color.lighter(0.02),
                            (false, false) => {
                                if minimal {
                                    Srgba::NONE
                                } else {
                                    base_color
                                }
                            }
                        };
                        let mut bg = cx.world_mut().get_mut::<BackgroundColor>(ent).unwrap();
                        bg.0 = color.into();
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
                    }),
                self.children.clone(),
                Spacer,
                Cond::new(
                    {
                        let no_caret = self.no_caret;
                        move |_cx| no_caret
                    },
                    || (),
                    || {
                        Icon::new("obsidian_ui://icons/chevron_down.png")
                            .color(Color::from(colors::DIM))
                            .style(|ss: &mut StyleBuilder| {
                                ss.margin_right(4);
                            })
                    },
                ),
                Cond::new(
                    move |cx| open.get(cx),
                    move || {
                        Portal::new(
                            Element::<NodeBundle>::new()
                                .style(style_menu_barrier)
                                .insert((
                                    On::<Pointer<Click>>::run(move |world: &mut World| {
                                        if !disabled.get(world) {
                                            let mut event = world
                                                .get_resource_mut::<ListenerInput<Pointer<Click>>>()
                                                .unwrap();
                                            event.stop_propagation();
                                            open.update(world, |mut state| {
                                                *state = !*state;
                                            });
                                        }
                                    }),
                                    ZIndex::Global(100),
                                ))
                                .children(popup.clone()),
                        )
                    },
                    || (),
                ),
            ))
    }
}

fn style_popup(ss: &mut StyleBuilder) {
    ss.background_color(colors::U1)
        .border_radius(4.0)
        .position(PositionType::Absolute)
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Column)
        .justify_content(ui::JustifyContent::FlexStart)
        .align_items(ui::AlignItems::Stretch)
        .border_color(Srgba::BLACK)
        .border(1)
        .padding((0, 2));
}

/// UI component representing the popup menu.
pub struct MenuPopup {
    /// The children of the popup.
    pub children: ChildArray,

    /// Additional styles to apply to the popup.
    pub style: StyleHandle,

    /// Whether to align the popup to the left or right side of the anchor.
    pub align: FloatAlign,

    /// Default side of the popup (top, bottom, left, right). Note that the popup will also
    /// automatically flip to the opposite side if it doesn't fit on the default side.
    pub side: FloatSide,
}

impl Default for MenuPopup {
    fn default() -> Self {
        Self {
            children: Default::default(),
            style: Default::default(),
            align: FloatAlign::Start,
            side: FloatSide::Bottom,
        }
    }
}

impl MenuPopup {
    /// Create a new menu popup.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the children of the popup.
    pub fn children<V: ChildViewTuple>(mut self, children: V) -> Self {
        self.children = children.to_child_array();
        self
    }

    /// Set additional styles to apply to the popup.
    pub fn style(mut self, style: StyleHandle) -> Self {
        self.style = style;
        self
    }

    /// Set the alignment of the popup.
    pub fn align(mut self, align: FloatAlign) -> Self {
        self.align = align;
        self
    }

    /// Set the default side of the popup.
    pub fn side(mut self, side: FloatSide) -> Self {
        self.side = side;
        self
    }
}

impl ViewTemplate for MenuPopup {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        // Adds a delay to ensure the menu items are created before setting focus.
        let state = cx.create_bistable_transition(Signal::Constant(true), 0.01);
        let context = cx.use_inherited_component::<MenuAnchor>().unwrap();
        let owner_id = cx.owner();

        Element::<NodeBundle>::new()
            .named("MenuPopup")
            .style((typography::text_default, style_popup, self.style.clone()))
            .insert((
                TabGroup {
                    order: 1,
                    modal: true,
                },
                Floating {
                    anchor: context.0,
                    position: vec![
                        FloatPosition {
                            side: self.side,
                            align: self.align,
                            stretch: false,
                            gap: 2.0,
                        },
                        FloatPosition {
                            side: self.side.mirror(),
                            align: self.align,
                            stretch: false,
                            gap: 2.0,
                        },
                    ],
                },
                On::<Pointer<Click>>::run(move |world: &mut World| {
                    let mut event = world
                        .get_resource_mut::<ListenerInput<Pointer<Click>>>()
                        .unwrap();
                    event.stop_propagation();
                }),
                On::<KeyPressEvent>::run(move |world: &mut World| {
                    let mut st: SystemState<(
                        ResMut<ListenerInput<KeyPressEvent>>,
                        ResMut<Focus>,
                        TabNavigation,
                    )> = SystemState::new(world);
                    let (mut event, mut focus, nav) = st.get_mut(world);
                    if !event.repeat {
                        match event.key_code {
                            KeyCode::Escape => {
                                event.stop_propagation();
                                world.send_event(MenuCloseEvent { target: owner_id });
                            }
                            KeyCode::ArrowUp => {
                                event.stop_propagation();
                                focus.0 = nav.navigate(focus.0, NavAction::Previous);
                            }
                            KeyCode::ArrowDown => {
                                event.stop_propagation();
                                focus.0 = nav.navigate(focus.0, NavAction::Next);
                            }
                            KeyCode::Home => {
                                event.stop_propagation();
                                focus.0 = nav.navigate(focus.0, NavAction::First);
                            }
                            KeyCode::End => {
                                event.stop_propagation();
                                focus.0 = nav.navigate(focus.0, NavAction::Last);
                            }
                            _ => {}
                        }
                    }
                }),
            ))
            .children(self.children.clone())
            .create_effect(move |cx, ent| {
                if state.get(cx) == BistableTransitionState::Entered {
                    let mut st: SystemState<(ResMut<Focus>, TabNavigation)> =
                        SystemState::new(cx.world_mut());
                    let (mut focus, nav) = st.get_mut(cx.world_mut());
                    focus.0 = nav.navigate(Some(ent), NavAction::First);
                }
            })
    }
}

fn style_menu_item(ss: &mut StyleBuilder) {
    ss.height(24)
        .display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .justify_content(ui::JustifyContent::FlexStart)
        .align_items(ui::AlignItems::Center)
        .padding((6, 0))
        .margin((2, 0));
}

/// UI component representing a menu item.
#[derive(Default)]
pub struct MenuItem {
    /// The label of the menu item.
    pub label: ChildArray,

    /// Additional styles to apply to the menu item.
    pub style: StyleHandle,

    /// Whether the menu item is checked.
    pub checked: Signal<bool>,

    /// Whether the menu item is disabled.
    pub disabled: Signal<bool>,

    /// Callback called when clicked
    pub on_click: Option<Callback>,
    // icon
    // shortcut
}

impl MenuItem {
    /// Create a new menu item.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the label of the menu item.
    pub fn label<V: ChildViewTuple>(mut self, label: V) -> Self {
        self.label = label.to_child_array();
        self
    }

    /// Set additional styles to apply to the menu item.
    pub fn style(mut self, style: StyleHandle) -> Self {
        self.style = style;
        self
    }

    /// Set the checked state of the menu item.
    pub fn checked(mut self, checked: impl IntoSignal<bool>) -> Self {
        self.checked = checked.into_signal();
        self
    }

    /// Set the disabled state of the menu item.
    pub fn disabled(mut self, disabled: impl IntoSignal<bool>) -> Self {
        self.disabled = disabled.into_signal();
        self
    }

    /// Set the callback to be called when the menu item is clicked.
    pub fn on_click(mut self, on_click: Callback) -> Self {
        self.on_click = Some(on_click);
        self
    }
}

impl ViewTemplate for MenuItem {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let id = cx.create_entity();
        let owner_id = cx.owner();
        let pressed = cx.create_mutable::<bool>(false);
        let hovering = cx.create_hover_signal(id);
        let focused = cx.create_focus_signal(id);

        let disabled = self.disabled;

        Element::<NodeBundle>::for_entity(id)
            .named("MenuItem")
            .style((style_menu_item, self.style.clone()))
            .insert((
                TabIndex(0),
                AccessibilityNode::from(NodeBuilder::new(Role::Button)),
                {
                    let on_click = self.on_click;
                    On::<Pointer<Click>>::run(move |world: &mut World| {
                        let mut st: SystemState<(EventWriter<MenuCloseEvent>, ResMut<Focus>)> =
                            SystemState::new(world);
                        if !disabled.get(world) {
                            let (mut writer, mut focus) = st.get_mut(world);
                            focus.0 = Some(id);
                            if let Some(on_click) = on_click {
                                writer.send(MenuCloseEvent { target: owner_id });
                                world.run_callback(on_click, ());
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
                    if !disabled.get(world) {
                        pressed.set(world, false);
                    }
                }),
                On::<KeyPressEvent>::run({
                    let on_click = self.on_click;
                    move |world: &mut World| {
                        if !disabled.get(world) {
                            let mut st: SystemState<(
                                ResMut<ListenerInput<KeyPressEvent>>,
                                EventWriter<MenuCloseEvent>,
                            )> = SystemState::new(world);
                            let (mut event, mut writer) = st.get_mut(world);
                            if !event.repeat
                                && (event.key_code == KeyCode::Enter
                                    || event.key_code == KeyCode::Space)
                            {
                                event.stop_propagation();
                                if let Some(on_click) = on_click {
                                    writer.send(MenuCloseEvent { target: owner_id });
                                    world.run_callback(on_click, ());
                                }
                            }
                        }
                    }
                }),
            ))
            .create_effect(move |cx, ent| {
                let is_pressed = pressed.get(cx);
                let is_hovering = hovering.get(cx);
                let is_focused = focused.get(cx);
                let color = match (is_pressed || is_focused, is_hovering) {
                    (true, true) => colors::U1.lighter(0.03),
                    (true, false) => colors::U1.lighter(0.02),
                    (false, true) => colors::U1.lighter(0.01),
                    (false, false) => Srgba::NONE,
                };
                let mut bg = cx.world_mut().get_mut::<BackgroundColor>(ent).unwrap();
                bg.0 = color.into();
            })
            .children(self.label.clone())
    }
}

fn style_menu_divider(ss: &mut StyleBuilder) {
    ss.height(1).background_color(Srgba::BLACK).margin((0, 2));
}

/// UI component representing a menu divider.
#[derive(Default)]
pub struct MenuDivider;

impl ViewTemplate for MenuDivider {
    fn create(&self, _cx: &mut Cx) -> impl IntoView {
        Element::<NodeBundle>::new()
            .named("MenuDivider")
            .style(style_menu_divider)
    }
}
