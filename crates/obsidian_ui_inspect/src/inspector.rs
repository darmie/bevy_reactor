use std::sync::Arc;

use bevy::{
    color::Color,
    ecs::reflect::AppTypeRegistry,
    reflect::{
        std_traits::ReflectDefault, DynamicEnum, DynamicTuple, OffsetAccess, ParsedPath,
        ReflectKind, ReflectRef, TypeInfo, VariantInfo,
    },
};
use bevy_reactor_core::*;
use bevy_reactor_style::*;
use bevy_reactor_view::*;
use obsidian_ui::{
    colors,
    controls::{Icon, MenuButton, MenuItem, MenuPopup, Spacer},
    floating::FloatAlign,
    size::Size,
};

use crate::{
    templates::inspector_panel::InspectorPanel, Inspectable, InspectableField,
    InspectorFactoryRegistry,
};

pub struct Inspector {
    // Reference to the entity being inspected
    target: Arc<dyn Inspectable>,
}

impl Inspector {
    pub fn new(target: Arc<dyn Inspectable>) -> Self {
        Self { target }
    }

    fn create_fields(&self, cx: &mut Cx, target: Arc<dyn Inspectable>) -> ViewRef {
        let field_type = cx.create_memo(move |cx| target.reflect(cx).reflect_kind());
        let target = self.target.clone();
        DynamicKeyed::new(
            move |cx| field_type.get(cx),
            move |ftype| match ftype {
                ReflectKind::Struct => StructInspector {
                    target: target.clone(),
                },
                _ => todo!(),
            },
        )
        .into_view()
    }
}

impl ViewTemplate for Inspector {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        InspectorPanel::new()
            .title((
                self.target.name(cx),
                Spacer,
                AddFieldsButton {
                    target: self.target.clone(),
                },
            ))
            .body(self.create_fields(cx, self.target.clone()))
            .expanded(true)
    }
}

struct StructInspector {
    target: Arc<dyn Inspectable>,
}

impl ViewTemplate for StructInspector {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let target = self.target.clone();
        // Get the memoized field names of the struct, minus missing optionals. This should
        // isolate the field editors from each other so that they don't constantly update.
        // We will still need to memoize the individual field values.
        let field_names = cx.create_memo(move |cx| {
            let ReflectRef::Struct(st) = target.reflect(cx).reflect_ref() else {
                panic!("Expected ReflectRef::Struct")
            };
            let num_fields = st.field_len();
            let mut names = Vec::with_capacity(num_fields);
            // Filter out field names for fields with a value of `None`.
            for findex in 0..num_fields {
                let field = st.field_at(findex).unwrap();
                if field.reflect_kind() == ReflectKind::Enum
                    && field
                        .reflect_type_path()
                        .starts_with("core::option::Option")
                {
                    let ReflectRef::Enum(enum_ref) = field.reflect_ref() else {
                        panic!("Expected ReflectRef::Enum");
                    };
                    if enum_ref.variant_name() != "None" {
                        names.push(st.name_at(findex).unwrap().to_string());
                    }
                } else {
                    names.push(st.name_at(findex).unwrap().to_string());
                }
            }
            names
        });
        let target = self.target.clone();
        For::each(
            move |cx| field_names.get_clone(cx).into_iter(),
            move |name| {
                let path = ParsedPath::parse(name).unwrap();
                let field = Arc::new(InspectableField {
                    root: target.clone(),
                    name: name.to_string(),
                    path: path.clone(),
                    container_path: path,
                    can_remove: false,
                });
                FieldInspector { field }.into_view()
            },
        )
    }
}

struct FieldInspector {
    field: Arc<InspectableField>,
}

impl ViewTemplate for FieldInspector {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let factories = cx.use_resource::<InspectorFactoryRegistry>();
        let field = self.field.clone();
        let Some(reflect) = field.reflect(cx) else {
            return ().into_view();
        };

        // If the field is Option<T>, and not None, then unwrap the value and inspect the
        // inner value.
        if reflect.reflect_kind() == ReflectKind::Enum
            && reflect
                .reflect_type_path()
                .starts_with("core::option::Option")
        {
            let ReflectRef::Enum(enum_ref) = reflect.reflect_ref() else {
                panic!("Expected ReflectRef::Enum");
            };
            if enum_ref.variant_name() != "None" {
                let mut path = field.path.clone();
                path.0.push(OffsetAccess {
                    access: bevy::reflect::Access::TupleIndex(0),
                    offset: None,
                });

                let access = Arc::new(InspectableField {
                    root: field.root.clone(),
                    name: field.name.clone(),
                    path,
                    container_path: field.path.clone(),
                    can_remove: true,
                });
                for factory in factories.0.iter().rev() {
                    if let Some(view_ref) = factory.create_inspector(cx, &access) {
                        return view_ref;
                    }
                }
            }
        } else {
            for factory in factories.0.iter().rev() {
                if let Some(view_ref) = factory.create_inspector(cx, &field) {
                    return view_ref;
                }
            }
        }

        // No inspector found, don't render anything. Note that default factory already
        // has a fallback, so this should never be reached.
        ().into_view()
    }
}

struct AddFieldsButton {
    target: Arc<dyn Inspectable>,
}

impl ViewTemplate for AddFieldsButton {
    fn create(&self, _cx: &mut Cx) -> impl IntoView {
        let target = self.target.clone();
        Dynamic::new(move |cx| match target.reflect(cx).reflect_ref() {
            ReflectRef::Struct(st) => {
                let num_fields = st.field_len();
                let mut items: Vec<ViewRef> = Vec::new();
                let registry = cx.world().resource::<AppTypeRegistry>().0.clone();
                for findex in 0..num_fields {
                    let field = st.field_at(findex).unwrap();
                    let name = st.name_at(findex).unwrap();
                    if field.reflect_kind() == ReflectKind::Enum
                        && field
                            .reflect_type_path()
                            .starts_with("core::option::Option")
                    {
                        let ReflectRef::Enum(enum_ref) = field.reflect_ref() else {
                            panic!("Expected ReflectRef::Enum");
                        };
                        let Some(TypeInfo::Enum(enum_info)) = field.get_represented_type_info()
                        else {
                            panic!("Expected TypeInfo::Enum");
                        };

                        if enum_ref.variant_name() == "None" {
                            let some_variant = enum_info.variant("Some").unwrap();
                            let VariantInfo::Tuple(tuple_info) = some_variant else {
                                panic!()
                            };
                            let some_field = tuple_info.field_at(0).unwrap();
                            let some_type_id = some_field.type_id();
                            let registry_lock = registry.read();
                            let some_type = registry_lock.get_type_info(some_type_id).unwrap();
                            let some_default =
                                registry_lock.get_type_data::<ReflectDefault>(some_type_id);
                            if some_default.is_some() {
                                items.push(
                                    AddStructFieldItem {
                                        field: Arc::new(InspectableField {
                                            root: target.clone(),
                                            name: name.to_string(),
                                            path: ParsedPath::parse(name).unwrap(),
                                            container_path: ParsedPath::parse(name).unwrap(),
                                            can_remove: false,
                                        }),
                                        // path: ParsedPath::parse(name).unwrap(),
                                        // name: name.to_string(),
                                    }
                                    .into_view(),
                                );
                            } else {
                                println!(
                                    "Can't find ReflectDefault for: {:?}",
                                    some_type.type_path()
                                );
                                // println!("Some default: {:?}", some_default.unwrap().default());
                            }

                            // let field_type = enum_ref.variant_type();
                            // let some_type = field_type.type_id();
                            // let ft = field_type.type_id();
                        }
                    }
                }

                if !items.is_empty() {
                    MenuButton::new()
                        .children(
                            Icon::new("obsidian_ui://icons/add_box.png")
                                .color(Color::from(colors::DIM))
                                .style(style_menu_icon),
                        )
                        .popup(
                            MenuPopup::new()
                                .align(FloatAlign::End)
                                .children(Fragment::from_slice(&items).into_view()),
                        )
                        .size(Size::Xxs)
                        .minimal(true)
                        .into_view()
                } else {
                    ().into_view()
                }
            }
            _ => {
                println!("Fallback: {}", target.reflect(cx).reflect_type_path());
                ().into_view()
            }
        })
    }
}

struct AddStructFieldItem {
    field: Arc<InspectableField>,
}

impl ViewTemplate for AddStructFieldItem {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let field = self.field.clone();
        let callback = cx.create_callback(move |cx, _| {
            let Some(field_reflect) = field.reflect(cx) else {
                return;
            };
            let Some(TypeInfo::Enum(enum_info)) = field_reflect.get_represented_type_info() else {
                panic!("Expected TypeInfo::Enum");
            };

            // let field = target.get_field(cx, &path);
            let registry = cx.world().resource::<AppTypeRegistry>().0.clone();
            let some_variant = enum_info.variant("Some").unwrap();
            let VariantInfo::Tuple(tuple_info) = some_variant else {
                panic!("Expected VariantInfo::Tuple");
            };
            let some_field = tuple_info.field_at(0).unwrap();
            let some_type_id = some_field.type_id();
            let registry_lock = registry.read();
            let some_type = registry_lock.get_type_info(some_type_id).unwrap();
            if some_type.is::<bool>() {
                // For Option<bool> we assume that the user wants a default of 'true', because
                // that's the most common use case. This is because for most fields, `Some(false)`
                // is the same as `None`.
                let mut data = DynamicTuple::default();
                data.insert_boxed(Box::new(true));
                let dynamic_enum = DynamicEnum::new("Some", data);
                field.set_value(cx, &dynamic_enum);
            } else {
                let some_default = registry_lock.get_type_data::<ReflectDefault>(some_type_id);
                if some_default.is_some() {
                    // The value that needs to get wrapped in `Some`.
                    let default = some_default.unwrap().default();
                    let mut data = DynamicTuple::default();
                    data.insert_boxed(default);
                    let dynamic_enum = DynamicEnum::new("Some", data);
                    field.set_value(cx, &dynamic_enum);
                } else {
                    println!("Can't find ReflectDefault for: {:?}", some_type.type_path());
                }
            }
        });
        MenuItem::new()
            .label(self.field.name.clone())
            .on_click(callback)
    }
}

fn style_menu_icon(ss: &mut StyleBuilder) {
    ss.margin((4, 0));
}
