use bevy::{prelude::*, ui};
use bevy_reactor_core::*;
use bevy_reactor_style::*;
use bevy_reactor_view::*;
use obsidian_ui::{
    colors,
    controls::{
        EdgeDisplay, GraphDisplay, InputTerminalDisplay, NodeDisplay, OutputTerminalDisplay,
        Slider, Swatch,
    },
};

#[derive(Clone, Debug, PartialEq, Component)]
pub struct NodePosition(pub Vec2);

#[derive(Clone, Debug, PartialEq, Component)]
pub struct NodeTitle(pub String);

#[derive(Clone, Debug, PartialEq, Component)]
pub struct NodeOutputs(pub Vec<Entity>);

#[derive(Clone, Debug, PartialEq, Component)]
pub struct NodeInputs(pub Vec<Entity>);

#[derive(Clone, Debug, PartialEq, Component)]
pub struct Edge {
    pub src: Entity,
    pub dst: Entity,
}

pub trait DataType {
    fn color(&self) -> Srgba;
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum DemoDataType {
    Float((f32, f32)),
    Rgb,
}

impl DataType for DemoDataType {
    fn color(&self) -> Srgba {
        match self {
            DemoDataType::Float(_) => colors::U4,
            DemoDataType::Rgb => colors::RESOURCE,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum DemoValueType {
    Float(f32),
    Rgb(Srgba),
}

#[derive(Debug, Component)]
pub struct InputTerminal<DataType> {
    pub label: String,
    pub data_type: DataType,
    pub connection: Option<Entity>,
}

#[derive(Debug, Component)]
pub struct InputTerminalValue<ValueType>(ValueType);

#[derive(Debug, Component)]
pub struct OutputTerminal<DataType, ValueType> {
    pub label: String,
    pub data_type: DataType,
    pub value: ValueType,
    pub connections: Vec<Entity>,
}

#[derive(Debug, Component)]
pub struct TerminalDisplay(pub Option<Entity>);

#[derive(Debug, Resource)]
pub struct DemoGraphRoot {
    nodes: Vec<Entity>,
    edges: Vec<Entity>,
}

impl FromWorld for DemoGraphRoot {
    fn from_world(world: &mut World) -> Self {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let node1 = world
            .spawn((
                NodePosition(Vec2::new(100., 100.)),
                NodeTitle("Node 1".to_string()),
            ))
            .id();
        nodes.push(node1);
        let outputs = vec![world
            .spawn((
                OutputTerminal::<DemoDataType, DemoValueType> {
                    label: "Color".to_string(),
                    data_type: DemoDataType::Rgb,
                    value: DemoValueType::Rgb(Srgba::new(1., 0., 0., 1.)),
                    connections: Vec::new(),
                },
                TerminalDisplay(None),
            ))
            .set_parent(node1)
            .id()];
        let edge_out1 = outputs[0];
        world.entity_mut(node1).insert(NodeOutputs(outputs));

        let inputs = vec![
            world
                .spawn((
                    InputTerminal::<DemoDataType> {
                        label: "Base".to_string(),
                        data_type: DemoDataType::Float((0., 100.)),
                        connection: None,
                    },
                    InputTerminalValue(DemoValueType::Float(50.)),
                    TerminalDisplay(None),
                ))
                .set_parent(node1)
                .id(),
            world
                .spawn((
                    InputTerminal::<DemoDataType> {
                        label: "Mask".to_string(),
                        data_type: DemoDataType::Float((0., 100.)),
                        connection: None,
                    },
                    InputTerminalValue(DemoValueType::Float(80.)),
                    TerminalDisplay(None),
                ))
                .set_parent(node1)
                .id(),
        ];
        world.entity_mut(node1).insert(NodeInputs(inputs));

        let node2 = world
            .spawn((
                NodePosition(Vec2::new(300., 220.)),
                NodeTitle("Node 2".to_string()),
            ))
            .id();
        nodes.push(node2);
        let outputs = vec![world
            .spawn((
                OutputTerminal::<DemoDataType, DemoValueType> {
                    label: "Color".to_string(),
                    data_type: DemoDataType::Rgb,
                    value: DemoValueType::Rgb(Srgba::new(1., 0., 0., 1.)),
                    connections: Vec::new(),
                },
                TerminalDisplay(None),
            ))
            .set_parent(node2)
            .id()];
        world.entity_mut(node2).insert(NodeOutputs(outputs));

        let inputs = vec![
            world
                .spawn((
                    InputTerminal::<DemoDataType> {
                        label: "A".to_string(),
                        data_type: DemoDataType::Float((0., 1.0)),
                        connection: None,
                    },
                    InputTerminalValue(DemoValueType::Float(0.3)),
                    TerminalDisplay(None),
                ))
                .set_parent(node2)
                .id(),
            world
                .spawn((
                    InputTerminal::<DemoDataType> {
                        label: "B".to_string(),
                        data_type: DemoDataType::Float((0., 1.0)),
                        connection: None,
                    },
                    InputTerminalValue(DemoValueType::Float(0.2)),
                    TerminalDisplay(None),
                ))
                .set_parent(node2)
                .id(),
        ];
        let edge_in1 = inputs[0];
        world.entity_mut(node2).insert(NodeInputs(inputs));

        let node3 = world
            .spawn((
                NodePosition(Vec2::new(400., 100.)),
                NodeTitle("Node 3".to_string()),
            ))
            .id();
        nodes.push(node3);
        let outputs = vec![world
            .spawn((
                OutputTerminal::<DemoDataType, DemoValueType> {
                    label: "Out".to_string(),
                    data_type: DemoDataType::Rgb,
                    value: DemoValueType::Rgb(Srgba::new(1., 0., 0., 1.)),
                    connections: Vec::new(),
                },
                TerminalDisplay(None),
            ))
            .set_parent(node3)
            .id()];
        world.entity_mut(node3).insert(NodeOutputs(outputs));

        let inputs = vec![world
            .spawn((
                InputTerminal::<DemoDataType> {
                    label: "In".to_string(),
                    data_type: DemoDataType::Float((0., 1.0)),
                    connection: None,
                },
                InputTerminalValue(DemoValueType::Float(0.3)),
                TerminalDisplay(None),
            ))
            .set_parent(node3)
            .id()];
        let edge_in2 = inputs[0];
        world.entity_mut(node3).insert(NodeInputs(inputs));

        edges.push(
            world
                .spawn(Edge {
                    src: edge_out1,
                    dst: edge_in1,
                })
                .id(),
        );
        edges.push(
            world
                .spawn(Edge {
                    src: edge_out1,
                    dst: edge_in2,
                })
                .id(),
        );

        Self { nodes, edges }
    }
}

pub struct NodeGraphDemo {}

fn style_node_graph(ss: &mut StyleBuilder) {
    ss.flex_grow(1.).border_left(1).border_color(Color::BLACK);
}

fn style_slider(ss: &mut StyleBuilder) {
    ss.min_width(150.);
}

fn style_input_label(ss: &mut StyleBuilder) {
    ss.display(ui::Display::Flex)
        .flex_direction(ui::FlexDirection::Row)
        .justify_content(ui::JustifyContent::FlexStart)
        .align_items(ui::AlignItems::Center)
        .min_height(20.);
}

impl ViewTemplate for NodeGraphDemo {
    fn create(&self, _cx: &mut Cx) -> impl IntoView {
        GraphDisplay::new()
            .children((
                For::each(
                    |cx| {
                        let graph = cx.use_resource::<DemoGraphRoot>();
                        graph.edges.clone().into_iter()
                    },
                    |id| EdgeTemplate { id: *id },
                ),
                For::each(
                    |cx| {
                        let graph = cx.use_resource::<DemoGraphRoot>();
                        graph.nodes.clone().into_iter()
                    },
                    |id| NodeTemplate { id: *id },
                ),
            ))
            .style(style_node_graph)
    }
}

pub struct NodeTemplate {
    id: Entity,
}

impl ViewTemplate for NodeTemplate {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let id = self.id;
        let position = cx.create_derived(move |cx| cx.use_component::<NodePosition>(id).unwrap().0);
        let title =
            cx.create_derived(move |cx| cx.use_component::<NodeTitle>(id).unwrap().0.clone());
        NodeDisplay::new()
            .position(position)
            .title(title)
            .children((
                For::each(
                    move |cx| {
                        cx.use_component::<NodeOutputs>(id)
                            .map_or(Vec::new(), |c| c.0.clone())
                            .into_iter()
                    },
                    |en| OutputTemplate { id: *en },
                ),
                For::each(
                    move |cx| {
                        cx.use_component::<NodeInputs>(id)
                            .map_or(Vec::new(), |c| c.0.clone())
                            .into_iter()
                    },
                    |en| InputTemplate { id: *en },
                ),
            ))
            .selected(Signal::Constant(false))
            .on_drag(cx.create_callback(move |cx, new_pos| {
                let mut entt = cx.world_mut().entity_mut(id);
                let mut pos = entt.get_mut::<NodePosition>().unwrap();
                pos.0 = new_pos;
            }))
    }
}

pub struct OutputTemplate {
    id: Entity,
}

impl ViewTemplate for OutputTemplate {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let display_id = cx.create_entity();
        // Make sure the display entity is updated when the terminal is updated.
        // This normally doesn't happen because the terminal display and the edge display are
        // created in the same tick.
        cx.world_mut().increment_change_tick();
        if let Some(mut td) = cx
            .world_mut()
            .entity_mut(self.id)
            .get_mut::<TerminalDisplay>()
        {
            td.0 = Some(display_id);
        }
        let id = self.id;
        let label = cx.create_derived(move |rcx| {
            rcx.use_component::<OutputTerminal<DemoDataType, DemoValueType>>(id)
                .unwrap()
                .label
                .clone()
        });
        let color = cx.create_derived(move |rcx| {
            rcx.use_component::<OutputTerminal<DemoDataType, DemoValueType>>(id)
                .unwrap()
                .data_type
                .color()
        });
        OutputTerminalDisplay {
            id: display_id,
            label: label.get_clone(cx),
            color: color.get(cx),
        }
    }
}

pub struct InputTemplate {
    id: Entity,
}

impl ViewTemplate for InputTemplate {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let display_id = cx.create_entity();
        if let Some(mut td) = cx
            .world_mut()
            .entity_mut(self.id)
            .get_mut::<TerminalDisplay>()
        {
            td.0 = Some(display_id);
        }
        let id = self.id;
        let label = cx.create_derived(move |rcx| {
            rcx.use_component::<InputTerminal<DemoDataType>>(id)
                .unwrap()
                .label
                .clone()
        });
        let is_connected = cx.create_derived(move |rcx| {
            rcx.use_component::<InputTerminal<DemoDataType>>(id)
                .unwrap()
                .connection
                .is_some()
        });
        let color = cx.create_derived(move |rcx| {
            rcx.use_component::<InputTerminal<DemoDataType>>(id)
                .unwrap()
                .data_type
                .color()
        });
        InputTerminalDisplay {
            id: display_id,
            color: color.get(cx),
            control: Cond::new(
                move |cx| is_connected.get(cx),
                move || {
                    Element::<NodeBundle>::new()
                        .style(style_input_label)
                        .children(label.clone())
                },
                move || {
                    Dynamic::new(move |cx| {
                        let data_type = cx
                            .use_component::<InputTerminal<DemoDataType>>(id)
                            .unwrap()
                            .data_type;
                        let label = cx
                            .use_component::<InputTerminal<DemoDataType>>(id)
                            .unwrap()
                            .label
                            .clone();
                        match data_type {
                            DemoDataType::Float(_) => FloatInputEdit { id, label }.into_view(),
                            DemoDataType::Rgb => {
                                // TODO: Replace with ColorInputEdit.
                                Swatch::new(Srgba::new(1., 0., 0., 1.)).into_view()
                            }
                        }
                    })
                },
            )
            .into_view(),
        }
    }
}

pub struct FloatInputEdit {
    id: Entity,
    label: String,
}

impl ViewTemplate for FloatInputEdit {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let id = self.id;
        let value = cx.create_derived(move |rcx| {
            if let DemoValueType::Float(value) = rcx
                .use_component::<InputTerminalValue<DemoValueType>>(id)
                .unwrap()
                .0
            {
                value
            } else {
                0.
            }
        });
        let min = cx.create_derived(move |rcx| {
            if let DemoDataType::Float((min, _)) = rcx
                .use_component::<InputTerminal<DemoDataType>>(id)
                .unwrap()
                .data_type
            {
                min
            } else {
                0.
            }
        });
        let max = cx.create_derived(move |rcx| {
            if let DemoDataType::Float((_, max)) = rcx
                .use_component::<InputTerminal<DemoDataType>>(id)
                .unwrap()
                .data_type
            {
                max
            } else {
                1.
            }
        });

        Slider::new()
            .value(value)
            .min(min)
            .max(max)
            .label(self.label.clone())
            .style(style_slider)
            .on_change(cx.create_callback(move |cx: &mut Cx, value: f32| {
                let mut entt = cx.world_mut().entity_mut(id);
                let mut val = entt.get_mut::<InputTerminalValue<DemoValueType>>().unwrap();
                val.0 = DemoValueType::Float(value);
            }))
    }
}

pub struct EdgeTemplate {
    id: Entity,
}

impl ViewTemplate for EdgeTemplate {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let edge = cx.use_component::<Edge>(self.id).unwrap().clone();
        let src_pos = cx.create_derived(move |cx| {
            let Some(TerminalDisplay(Some(display_id))) =
                cx.use_component::<TerminalDisplay>(edge.src)
            else {
                return Vec2::default();
            };
            let Some(node_rect) = get_relative_rect(cx, *display_id, 3) else {
                return Vec2::default();
            };
            Vec2::new(node_rect.max.x, node_rect.min.y.lerp(node_rect.max.y, 0.5))
        });
        let dst_pos = cx.create_derived(move |cx| {
            let Some(TerminalDisplay(Some(display_id))) =
                cx.use_component::<TerminalDisplay>(edge.dst)
            else {
                return Vec2::default();
            };
            let Some(node_rect) = get_relative_rect(cx, *display_id, 3) else {
                return Vec2::default();
            };
            Vec2::new(node_rect.min.x, node_rect.min.y.lerp(node_rect.max.y, 0.5))
        });
        EdgeDisplay { src_pos, dst_pos }
    }
}

fn get_relative_rect(cx: &Rcx, id: Entity, levels: usize) -> Option<Rect> {
    cx.world().get_entity(id)?;
    let node = cx.use_component::<Node>(id)?;
    let transform = cx.use_component::<GlobalTransform>(id)?;
    let mut rect = node.logical_rect(transform);
    let mut current = id;
    for _ in 0..levels {
        if let Some(parent) = cx.use_component::<Parent>(current) {
            current = parent.get();
        } else {
            return None;
        }
    }
    let node = cx.use_component::<Node>(current)?;
    let transform = cx.use_component::<GlobalTransform>(current)?;
    let ancestor_rect = node.logical_rect(transform);
    rect.min -= ancestor_rect.min;
    rect.max -= ancestor_rect.min;
    Some(rect)
}
