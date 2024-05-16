use bevy::prelude::*;
use bevy_mod_picking::{focus::HoverMap, pointer::PointerId};
use bevy::render::render_resource::Extent3d;
use bevy::utils::HashSet;
use bevy_reactor_core::{ReactionCell, TrackingScope, TrackingScopeTracing, Hovering};

use bevy_reactor_style::{InheritableFontStyles, TextStyleChanged};
use bevy_reactor_view::{CompositorCamera, DisplayNodeChanged, ViewHandle, ViewRoot};


/// System that initializes any views that have been added.
pub(crate) fn build_added_view_roots(world: &mut World) {
    // Need to copy query result to avoid double-borrow of world.
    let mut roots = world.query_filtered::<(Entity, &mut ViewRoot), Added<ViewRoot>>();
    let roots_copy: Vec<Entity> = roots.iter(world).map(|(e, _)| e).collect();
    for root_entity in roots_copy.iter() {
        let Ok((_, root)) = roots.get(world, *root_entity) else {
            continue;
        };
        let inner = root.0.clone();
        inner.lock().unwrap().build(*root_entity, world);
    }
}


/// System that looks for changed child views and replaces the parent's child nodes.
pub(crate)  fn attach_child_views(world: &mut World) {
    let mut query = world.query_filtered::<Entity, With<DisplayNodeChanged>>();
    let query_copy = query.iter(world).collect::<Vec<Entity>>();
    for entity in query_copy {
        world.entity_mut(entity).remove::<DisplayNodeChanged>();
        let mut e = entity;
        let mut finished = false;
        loop {
            if let Some(handle) = world.entity(e).get::<ViewHandle>() {
                let inner = handle.0.clone();
                if inner.lock().unwrap().children_changed(e, world) {
                    finished = true;
                    break;
                }
            }

            if let Some(handle) = world.entity(e).get::<ViewRoot>() {
                let inner = handle.0.clone();
                if inner.lock().unwrap().children_changed(e, world) {
                    finished = true;
                    break;
                }
            }

            e = match world.entity(e).get::<Parent>() {
                Some(parent) => parent.get(),
                None => {
                    break;
                }
            };
        }

        if !finished {
            warn!("DisplayNodeChanged not handled.");
            e = entity;
            loop {
                if let Some(name) = world.entity(e).get::<Name>() {
                    println!("* Entity: {:?}", name);
                } else {
                    println!("* Entity: {:?}", e);
                }
                e = match world.entity(e).get::<Parent>() {
                    Some(parent) => parent.get(),
                    None => {
                        break;
                    }
                };
            }
        }
    }
}

// Hover change detection system
// Note: previously this was implemented as a Reaction, however it was reacting every frame
// because HoverMap is mutated every frame regardless of whether or not it changed.
pub(crate) fn update_hover_states(
    hover_map: Option<Res<HoverMap>>,
    mut hovers: Query<(Entity, &mut Hovering)>,
    parent_query: Query<&Parent>,
) {
    let Some(hover_map) = hover_map else { return };
    let hover_set = hover_map.get(&PointerId::Mouse);
    for (entity, mut hoverable) in hovers.iter_mut() {
        let is_hovering = match hover_set {
            Some(map) => map
                .iter()
                .any(|(ha, _)| parent_query.iter_ancestors(*ha).any(|e| e == entity)),
            None => false,
        };
        if hoverable.0 != is_hovering {
            hoverable.0 = is_hovering;
        }
    }
}

// Text system 
pub(crate) fn update_text_styles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Text), With<TextStyleChanged>>,
    inherited: Query<&InheritableFontStyles>,
    parents: Query<&Parent>,
    server: Res<AssetServer>,
) {
    for (entity, mut text) in query.iter_mut() {
        let mut styles = InheritableFontStyles::default();

        // Search parents for inherited styles.
        let mut ancestor = entity;
        loop {
            if styles.is_final() {
                break;
            }
            if let Ok(inherited_styles) = inherited.get(ancestor) {
                styles.merge(inherited_styles);
                if styles.is_final() {
                    break;
                }
            }
            if let Ok(parent) = parents.get(ancestor) {
                ancestor = parent.get();
            } else {
                break;
            }
        }

        // If we have a font handle, but it's not ready, then skip this update.
        if let Some(ref handle) = styles.font {
            match server.load_state(handle) {
                bevy::asset::LoadState::Loaded => {}
                _ => {
                    continue;
                }
            }
        }

        let style = TextStyle {
            font: styles.font.unwrap_or_default(),
            font_size: styles.font_size.unwrap_or(12.),
            color: styles.color.unwrap_or(Color::WHITE),
        };

        for section in text.sections.iter_mut() {
            section.style = style.clone();
        }
        commands.entity(entity).remove::<TextStyleChanged>();
    }
}

// Compositor system
pub(crate) fn update_compositor_size(
    query_camera: Query<(Entity, &Camera), With<CompositorCamera>>,
    query_children: Query<(&Node, &GlobalTransform, &TargetCamera)>,
    mut images: ResMut<Assets<Image>>,
) {
    for (camera_entity, camera) in query_camera.iter() {
        let image = images.get_mut(camera.target.as_image().unwrap()).unwrap();
        let mut size = Extent3d {
            width: 16,
            height: 16,
            ..Extent3d::default()
        };

        for (node, transform, target) in query_children.iter() {
            let target = target.0;
            if target == camera_entity {
                let rect = node.logical_rect(transform);
                size.width = size.width.max(rect.max.x.ceil() as u32);
                size.height = size.height.max(rect.max.y.ceil() as u32);
            }
        }

        if image.width() != size.width || image.height() != size.height {
            image.resize(size);
        }
    }
}

/// Run reactions whose dependencies have changed.
pub(crate) fn run_reactions(world: &mut World) {
    let mut scopes = world.query::<(Entity, &mut TrackingScope)>();
    let mut changed = HashSet::<Entity>::default();
    let tick = world.change_tick();
    for (entity, scope) in scopes.iter(world) {
        if scope.dependencies_changed(world, tick) {
            changed.insert(entity);
        }
    }

    // Record the changed entities for debugging purposes.
    if let Some(mut tracing) = world.get_resource_mut::<TrackingScopeTracing>() {
        tracing.0 = changed.iter().copied().collect();
    }

    for scope_entity in changed.iter() {
        // Call registered cleanup functions
        let mut cleanups = match scopes.get_mut(world, *scope_entity) {
            Ok((_, mut scope)) => std::mem::take(&mut scope.cleanups),
            Err(_) => Vec::new(),
        };
        for cleanup_fn in cleanups.drain(..) {
            cleanup_fn(world);
        }

        // Run the reaction
        let mut next_scope = TrackingScope::new(tick);
        if let Some(mut entt) = world.get_entity_mut(*scope_entity) {
            if let Some(view_handle) = entt.get_mut::<ViewHandle>() {
                let inner = view_handle.0.clone();
                inner
                    .lock()
                    .unwrap()
                    .react(*scope_entity, world, &mut next_scope);
            } else if let Some(reaction) = entt.get_mut::<ReactionCell>() {
                let inner = reaction.0.clone();
                inner
                    .lock()
                    .unwrap()
                    .react(*scope_entity, world, &mut next_scope);
            }
        }

        // Replace deps and cleanups in the current scope with the next scope.
        if let Ok((_, mut scope)) = scopes.get_mut(world, *scope_entity) {
            // Swap the scopes so that the next scope becomes the current scope.
            // The old scopes will be dropped at the end of the loop block.
            scope.take_deps(&mut next_scope);
            scope.tick = tick;
        }
    }
}
