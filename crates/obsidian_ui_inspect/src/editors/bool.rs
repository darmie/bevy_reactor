use bevy::prelude::*;
use bevy_reactor_view::*;
use bevy_reactor_core::*;
use bevy_reactor_style::*;
use obsidian_ui::controls::Checkbox;

use crate::{templates::field_label::FieldLabel, InspectableField};

pub struct FieldEditBool(pub(crate) InspectableField);

impl ViewTemplate for FieldEditBool {
    fn create(&self, cx: &mut Cx) -> impl IntoView {
        let field = self.0.clone();
        let is_checked = cx.create_memo(move |cx| {
            if let Some(value) = field.reflect(cx) {
                if value.is::<bool>() {
                    return *value.downcast_ref::<bool>().unwrap();
                }
            }
            false
        });

        let field = self.0.clone();
        Fragment::new((
            FieldLabel {
                field: field.clone(),
            },
            Checkbox {
                checked: is_checked,
                on_change: Some(cx.create_callback(move |cx: &mut Cx, value: bool| {
                    field.set_value(cx, value.as_reflect());
                })),
                style: StyleHandle::new(|ss: &mut StyleBuilder| {
                    ss.justify_self(JustifySelf::Start);
                }),
                ..default()
            },
        ))
    }
}
