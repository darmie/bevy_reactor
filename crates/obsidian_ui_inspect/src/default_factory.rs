use crate::{
    editors::{
        bool::FieldEditBool, color::FieldEditSrgba, fallback::FieldEditFallback,
        list::FieldEditList,
    },
    templates::{field_label::FieldLabel, field_readonly_value::FieldReadonlyValue},
    InspectableField, InspectorFactory,
};
use bevy::reflect::ReflectRef;
use bevy_reactor_core::*;
use bevy_reactor_view::*;

#[derive(Default)]
pub struct DefaultInspectorFactory;

impl InspectorFactory for DefaultInspectorFactory {
    fn create_inspector(&self, cx: &Cx, field: &InspectableField) -> Option<ViewRef> {
        let reflect = field.reflect(cx)?;
        match reflect.reflect_ref() {
            ReflectRef::Struct(s) => {
                match s.reflect_type_path() {
                    "bevy_color::srgba::Srgba" => Some(
                        FieldEditSrgba {
                            field: field.clone(),
                        }
                        .into_view(),
                    ),

                    _ => Some(FieldEditFallback(field.clone()).into_view()),
                }
                // println!("Struct: {}", s.reflect_type_path());
                // views.push(
                //     InspectorFieldLabel {
                //         children: (
                //             name,
                //             Spacer,
                //             // Button {
                //             //     children: Element::<NodeBundle>::new()
                //             //         .with_styles(style_close_icon_small)
                //             //         .into(),
                //             //     size: Size::Xxxs,
                //             //     minimal: true,
                //             //     ..default()
                //             // },
                //         )
                //             .to_ref(),
                //         ..default()
                //     }
                //     .to_ref(),
                // );
                // views.push(
                //     InspectorFieldReadonlyValue {
                //         children: "Struct:TODO".into(),
                //         ..default()
                //     }
                //     .to_ref(),
                // );
            }
            ReflectRef::TupleStruct(_) => Some(
                Fragment::new((
                    FieldLabel {
                        field: field.clone(),
                    },
                    FieldReadonlyValue::new().children("TupleStruct:TODO"),
                ))
                .into_view(),
            ),
            ReflectRef::Tuple(_) => Some(
                Fragment::new((
                    FieldLabel {
                        field: field.clone(),
                    },
                    FieldReadonlyValue::new().children("Tuple:TODO"),
                ))
                .into_view(),
            ),
            ReflectRef::List(_) => Some(FieldEditList(field.clone()).into_view()),
            ReflectRef::Array(_) => Some(
                Fragment::new((
                    FieldLabel {
                        field: field.clone(),
                    },
                    FieldReadonlyValue::new().children("Array:TODO"),
                ))
                .into_view(),
            ),
            ReflectRef::Map(_) => Some(
                Fragment::new((
                    FieldLabel {
                        field: field.clone(),
                    },
                    FieldReadonlyValue::new().children("Map:TODO"),
                ))
                .into_view(),
            ),
            ReflectRef::Enum(_) => Some(
                Fragment::new((
                    FieldLabel {
                        field: field.clone(),
                    },
                    FieldReadonlyValue::new().children("Enum:TODO"),
                ))
                .into_view(),
            ),
            ReflectRef::Value(v) => match v.reflect_type_path() {
                "bool" => Some(FieldEditBool(field.clone()).into_view()),

                _ => Some(FieldEditFallback(field.clone()).into_view()),
            },
        }
    }
}
