use bevy::{
    prelude::*,
    render::{
        camera::RenderTarget,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
    },
};

use crate::{
    node_span::NodeSpan, style::ApplyStylesEffect, view::View, ChildView, ChildViewTuple,
    DespawnScopes, DisplayNodeChanged, EffectTarget, EntityEffect, IntoView, StyleTuple,
    TrackingScope, ViewRef, WithStyles,
};

/// Marker component which indicates that the entity is a camera for the compositor.
#[derive(Component)]
pub(crate) struct CompositorCamera;

/// A `Compositor` is a UI element which renders its children to an offscreen buffer, then
/// displays that buffer as a single element. This can be used for things like animating
/// opacity.
pub struct Compositor {
    children: Vec<ChildView>,
    camera: Option<Entity>,
    image: Option<Handle<Image>>,
    image_entity: Option<Entity>,

    /// List of effects to be added to the image entity.
    effects: Vec<Box<dyn EntityEffect>>,
}

impl Compositor {
    /// Construct a new `Compositor`.
    pub fn new<V: ChildViewTuple>(views: V) -> Self {
        let child_views = views.to_child_array().0;
        Self {
            children: child_views
                .iter()
                .map(|v| ChildView {
                    view: v.clone(),
                    entity: None,
                })
                .collect(),
            camera: None,
            image: None,
            image_entity: None,
            effects: Vec::new(),
        }
    }

    fn attach_children(&self, world: &mut World) {
        let mut count: usize = 0;
        for child in self.children.iter() {
            count += child.view.nodes().count();
        }

        let mut flat: Vec<Entity> = Vec::with_capacity(count);
        for child in self.children.iter() {
            child.view.nodes().flatten(&mut flat);
        }

        for child in flat.iter() {
            world
                .entity_mut(*child)
                .insert(TargetCamera(self.camera.unwrap()));
        }
    }
}

impl View for Compositor {
    fn nodes(&self) -> NodeSpan {
        NodeSpan::Node(self.image_entity.unwrap())
    }

    fn build(&mut self, view_entity: Entity, world: &mut World) {
        assert!(self.image_entity.is_none());
        world.entity_mut(view_entity).insert(Name::new("Portal"));
        // Build child nodes.
        for child in self.children.iter_mut() {
            child.entity = Some(ViewRef::spawn(&child.view, view_entity, world));
        }

        // Create offscreen buffer. Start with a default size, will resize later.
        let size = Extent3d {
            width: 16,
            height: 16,
            ..Extent3d::default()
        };

        let mut image = Image {
            texture_descriptor: TextureDescriptor {
                label: None,
                size,
                dimension: TextureDimension::D2,
                format: TextureFormat::Bgra8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: TextureUsages::TEXTURE_BINDING
                    | TextureUsages::COPY_DST
                    | TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            },
            ..Image::default()
        };
        image.resize(size);

        let image_handle = world
            .get_resource_mut::<Assets<Image>>()
            .unwrap()
            .add(image);

        // Create the entity that will display the image on the main UI camera.
        let image_entity = world
            .spawn((ImageBundle {
                image: UiImage::new(image_handle.clone()),
                style: Style {
                    width: Val::Auto,
                    height: Val::Auto,
                    ..default()
                },
                ..default()
            },))
            .id();

        // Create a 2d camera which will render the children to the offscreen buffer.
        self.camera = Some(
            world
                .spawn((
                    Camera2dBundle {
                        camera: Camera {
                            order: -1,
                            clear_color: ClearColorConfig::Custom(Color::default()),
                            target: RenderTarget::Image(image_handle.clone()),
                            ..Camera::default()
                        },

                        ..default()
                    },
                    CompositorCamera,
                ))
                .id(),
        );
        self.image = Some(image_handle);
        self.image_entity = Some(image_entity);

        // Insert components from effects.
        if !self.effects.is_empty() {
            let mut tracking = TrackingScope::new(world.change_tick());
            for effect in self.effects.iter_mut() {
                effect.start(view_entity, image_entity, world, &mut tracking);
            }
            world.entity_mut(view_entity).insert(tracking);
        }

        world.entity_mut(view_entity).insert(DisplayNodeChanged);
    }

    fn raze(&mut self, view_entity: Entity, world: &mut World) {
        // Raze all child views
        for child in self.children.iter_mut() {
            child.view.raze(child.entity.unwrap(), world);
            child.entity = None;
            // Child raze() will despawn itself.
        }

        world.despawn_owned_recursive(view_entity);
        world.despawn(self.image_entity.unwrap());
        world.despawn(self.camera.unwrap());
        self.image = None;
    }

    fn children_changed(&mut self, _view_entity: Entity, world: &mut World) -> bool {
        self.attach_children(world);
        true
    }
}

impl EffectTarget for Compositor {
    fn add_effect(&mut self, effect: Box<dyn EntityEffect>) {
        self.effects.push(effect);
    }
}

impl IntoView for Compositor {
    fn into_view(self) -> ViewRef {
        ViewRef::new(self)
    }
}

impl WithStyles for Compositor {
    fn style<S: StyleTuple + 'static>(mut self, styles: S) -> Self {
        self.add_effect(Box::new(ApplyStylesEffect { styles }));
        self
    }
}

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
