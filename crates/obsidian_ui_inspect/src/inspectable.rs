use core::panic;
use std::sync::Arc;

use bevy::{
    prelude::*,
    reflect::{DynamicEnum, DynamicVariant, ParsedPath, ReflectPathError},
};
use bevy_reactor_core::*;

/// Trait that represents an item that can be inspected
#[allow(unused_variables)]
pub trait Inspectable: Send + Sync {
    /// The name of the item being inspected
    fn name(&self, cx: &Cx) -> String;

    /// The reflect data for the item being inspected
    fn reflect<'a>(&self, cx: &'a Cx) -> &'a dyn Reflect;

    /// The reflect data for a field within the item.
    fn reflect_field<'a>(&self, cx: &'a Cx, path: &ParsedPath) -> Option<&'a dyn Reflect>;

    /// Update a field within the item
    fn set_field(&self, cx: &mut Cx, path: &ParsedPath, value: &dyn Reflect);

    /// Apply a closure to a field within the item
    fn update_field(&self, cx: &mut Cx, path: &ParsedPath, f: &dyn Fn(&mut dyn Reflect));
}

/// A resource that can be inspected
pub struct InspectableResource<T: Resource + Reflect> {
    marker: std::marker::PhantomData<T>,
}

impl<T: Resource + Reflect> Default for InspectableResource<T> {
    fn default() -> Self {
        Self {
            marker: std::marker::PhantomData,
        }
    }
}

impl<T: Resource + Reflect> Inspectable for InspectableResource<T> {
    fn name(&self, cx: &Cx) -> String {
        let res = cx.use_resource::<T>();
        res.reflect_short_type_path().to_string()
    }

    fn reflect<'a>(&self, cx: &'a Cx) -> &'a dyn Reflect {
        cx.use_resource::<T>().as_reflect()
    }

    fn reflect_field<'a>(&self, cx: &'a Cx, path: &ParsedPath) -> Option<&'a dyn Reflect> {
        let res = cx.use_resource::<T>();
        match res.reflect_path(path) {
            Ok(result) => Some(result),
            Err(ReflectPathError::InvalidAccess(_)) => None,
            Err(err) => panic!("{:?}", err),
        }
    }

    fn set_field(&self, cx: &mut Cx, path: &ParsedPath, value: &dyn Reflect) {
        let mut res = cx.world_mut().get_resource_mut::<T>().unwrap();
        res.reflect_path_mut(path).unwrap().apply(value);
    }

    fn update_field(&self, cx: &mut Cx, path: &ParsedPath, f: &dyn Fn(&mut dyn Reflect)) {
        let mut res = cx.world_mut().get_resource_mut::<T>().unwrap();
        f(res.reflect_path_mut(path).unwrap());
    }
}

/// A reference to a field within an `Inspectable`. This contains information needed to
/// get and set the field as well as query it's type.
#[derive(Clone)]
pub struct InspectableField {
    pub(crate) root: Arc<dyn Inspectable>,
    pub(crate) name: String,
    pub(crate) path: ParsedPath,
    pub(crate) container_path: ParsedPath,
    pub(crate) can_remove: bool,
}

impl InspectableField {
    /// Return the name of this field.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the reflected value of the field.
    pub fn reflect<'a>(&self, cx: &'a Cx) -> Option<&'a dyn Reflect> {
        self.root.reflect_field(cx, &self.path)
    }

    /// Update the value of the field
    pub fn set_value(&self, cx: &mut Cx, value: &dyn Reflect) {
        self.root.set_field(cx, &self.path, value);
    }

    /// Whether the item can be removed (in other words, is it optional or an array element)
    pub fn can_remove(&self) -> bool {
        self.can_remove
    }

    /// Use a closure to modify the reflected field data.
    pub fn update(&self, cx: &mut Cx, f: &dyn Fn(&mut dyn Reflect)) {
        self.root.update_field(cx, &self.path, f);
    }

    /// Remove the value from the parent
    pub fn remove(&self, cx: &mut Cx) {
        let Some(field) = self.root.reflect_field(cx, &self.container_path) else {
            return;
        };
        match field.get_represented_type_info().unwrap() {
            bevy::reflect::TypeInfo::Struct(_) => todo!(),
            bevy::reflect::TypeInfo::TupleStruct(_) => todo!(),
            bevy::reflect::TypeInfo::Tuple(_) => todo!(),
            bevy::reflect::TypeInfo::List(_) => todo!(),
            bevy::reflect::TypeInfo::Array(_) => todo!(),
            bevy::reflect::TypeInfo::Map(_) => todo!(),
            bevy::reflect::TypeInfo::Enum(_enum_ref) => {
                if field
                    .reflect_type_path()
                    .starts_with("core::option::Option")
                {
                    let dynamic_enum = DynamicEnum::new("None", DynamicVariant::Unit);
                    self.root.set_field(cx, &self.container_path, &dynamic_enum);
                } else {
                    panic!("Can't remove non-optional field");
                }
            }
            bevy::reflect::TypeInfo::Value(_) => todo!(),
        }
    }
}
