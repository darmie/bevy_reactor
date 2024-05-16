use std::f32::consts::PI;

use bevy::color::{LinearRgba, Luminance};
use bevy::prelude::*;
use bevy_mod_picking::{
    backend::ray::{RayId, RayMap},
    backends::raycast::RaycastPickable,
    prelude::*,
};
use bevy_reactor_core::*;
use bevy_reactor_view::*;
use bevy_reactor_overlays::{OverlayShape, PolygonOptions, StrokeMarker};
use obsidian_ui::colors;

#[derive(Default)]
pub struct TransformOverlay {
    /// Target entity to drag
    pub target: Signal<Option<Entity>>,
    /// Callback called when dragged. The argument is the new position of the object.
    pub on_change: Option<Callback<Vec3>>,
}

#[derive(Clone, PartialEq, Default, Copy)]
struct DragState {
    target_origin: Vec3,
    drag_origin: Vec3,
}

impl ViewTemplate for TransformOverlay {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let target_entity = self.target;
        let target_position = cx.create_derived(move |rcx| {
            if let Some(target) = target_entity.get(rcx) {
                if let Some(transform) = rcx.use_component::<GlobalTransform>(target) {
                    let mut trans = Transform::from_translation(transform.translation());
                    trans.rotate_local_x(-PI * 0.5);
                    return trans;
                }
            }
            Transform::default()
        });

        // Need to spawn the entity ids early so that we can test for hovering.
        let x_arrow = cx.create_entity();
        let y_arrow = cx.create_entity();

        let x_arrow_hovering = cx.create_hover_signal(x_arrow);
        let y_arrow_hovering = cx.create_hover_signal(y_arrow);

        let x_arrow_color: Signal<LinearRgba> = cx.create_derived(move |cx| {
            if x_arrow_hovering.get(cx) {
                colors::X_RED.lighter(0.1).into()
            } else {
                colors::X_RED.into()
            }
        });

        let y_arrow_color: Signal<LinearRgba> = cx.create_derived(move |cx| {
            if y_arrow_hovering.get(cx) {
                colors::Y_GREEN.lighter(0.1).into()
            } else {
                colors::Y_GREEN.into()
            }
        });

        let on_change = self.on_change;

        let drag_state = cx.create_mutable::<DragState>(DragState::default());

        // Using a portal here to suppress the error message about no parent.
        Portal::new(Cond::new(
            move |cx| target_entity.get(cx).is_some(),
            move || {
                OverlayShape::new(|_cx, sb| {
                    sb.with_stroke_width(0.2)
                        .stroke_rect(Rect::from_center_size(Vec2::new(0., 0.), Vec2::new(2., 2.)));
                })
                .with_transform_signal(target_position)
                .with_pickable(true)
                .children((
                    OverlayShape::for_entity(x_arrow, |_cx, sb| {
                        sb.with_stroke_width(0.3)
                            .stroke_polygon(
                                &[Vec2::new(1.2, 0.), Vec2::new(2., 0.)],
                                PolygonOptions {
                                    end_marker: StrokeMarker::Arrowhead,
                                    ..default()
                                },
                            )
                            .stroke_polygon(
                                &[Vec2::new(-1.2, 0.), Vec2::new(-2., 0.)],
                                PolygonOptions {
                                    end_marker: StrokeMarker::Arrowhead,
                                    ..default()
                                },
                            );
                    })
                    .with_color_signal(x_arrow_color)
                    .with_pickable(true)
                    .insert((
                        On::<Pointer<DragStart>>::run(move |world: &mut World| {
                            let mut event = world
                                .get_resource_mut::<ListenerInput<Pointer<DragStart>>>()
                                .unwrap();
                            event.stop_propagation();
                            let target_origin = target_position.get(world).translation;
                            let drag_origin = get_intersect_point(world, target_origin);
                            drag_state.set(
                                world,
                                DragState {
                                    target_origin,
                                    drag_origin,
                                },
                            );
                        }),
                        On::<Pointer<Drag>>::run(move |world: &mut World| {
                            let mut event = world
                                .get_resource_mut::<ListenerInput<Pointer<Drag>>>()
                                .unwrap();
                            event.stop_propagation();
                            if let Some(on_change) = on_change {
                                let ds = drag_state.get(world);
                                let new_pos = get_intersect_point(world, ds.target_origin);
                                let change = Vec3::new(new_pos.x - ds.drag_origin.x, 0., 0.);
                                world.run_callback(on_change, ds.target_origin + change);
                            }
                        }),
                    )),
                    OverlayShape::for_entity(y_arrow, |_cx, sb| {
                        sb.with_stroke_width(0.3)
                            .stroke_polygon(
                                &[Vec2::new(0., 1.2), Vec2::new(0., 2.)],
                                PolygonOptions {
                                    end_marker: StrokeMarker::Arrowhead,
                                    ..default()
                                },
                            )
                            .stroke_polygon(
                                &[Vec2::new(0., -1.2), Vec2::new(0., -2.)],
                                PolygonOptions {
                                    end_marker: StrokeMarker::Arrowhead,
                                    ..default()
                                },
                            );
                    })
                    .with_color_signal(y_arrow_color)
                    .with_pickable(true)
                    .insert((
                        On::<Pointer<DragStart>>::run(move |world: &mut World| {
                            let mut event = world
                                .get_resource_mut::<ListenerInput<Pointer<DragStart>>>()
                                .unwrap();
                            event.stop_propagation();
                            let target_origin = target_position.get(world).translation;
                            let drag_origin = get_intersect_point(world, target_origin);
                            drag_state.set(
                                world,
                                DragState {
                                    target_origin,
                                    drag_origin,
                                },
                            );
                        }),
                        On::<Pointer<Drag>>::run(move |world: &mut World| {
                            let mut event = world
                                .get_resource_mut::<ListenerInput<Pointer<Drag>>>()
                                .unwrap();
                            event.stop_propagation();
                            if let Some(on_change) = on_change {
                                let ds = drag_state.get(world);
                                let new_pos = get_intersect_point(world, ds.target_origin);
                                let change = Vec3::new(0., 0., new_pos.z - ds.drag_origin.z);
                                world.run_callback(on_change, ds.target_origin + change);
                            }
                        }),
                    )),
                ))
            },
            || (),
        ))
    }
}

fn get_intersect_point(world: &mut World, target_origin: Vec3) -> Vec3 {
    let camera_entity = world
        .query_filtered::<Entity, (With<Camera>, With<RaycastPickable>)>()
        .single(world);
    let ray_map = world.get_resource::<RayMap>().unwrap();
    let ray = ray_map
        .map()
        .get(&RayId::new(camera_entity, PointerId::Mouse))
        .unwrap();
    let intersect = ray
        .intersect_plane(target_origin, InfinitePlane3d::new(Vec3::new(0., 1., 0.)))
        .unwrap();
    ray.origin + ray.direction * intersect
}
