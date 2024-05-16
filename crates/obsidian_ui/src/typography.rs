use bevy_reactor_style::{StyleBuilder, StyleBuilderFont};

/// Default text style for UI.
pub fn text_default(ss: &mut StyleBuilder) {
    ss.font("obsidian_ui://fonts/Open_Sans/static/OpenSans-Medium.ttf")
        .font_size(16);
}
