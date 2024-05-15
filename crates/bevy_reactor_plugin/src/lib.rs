use bevy::prelude::*;

use bevy_reactor_core::{
    update_hover_states, run_reactions
};

use bevy_reactor_view::{attach_child_views, build_added_view_roots};
use bevy_reactor_view::{update_text_styles, update_compositor_size,};

/// Plugin that adds the reactive UI system to the app.
pub struct ReactorPlugin;

impl Plugin for ReactorPlugin {
    fn build(&self, app: &mut App) {
        app
            //.register_asset_loader(TextureAtlasLoader)
            .add_systems(
                Update,
                (
                    (
                        build_added_view_roots,
                        run_reactions,
                        attach_child_views,
                        update_text_styles,
                    )
                        .chain(),
                    update_hover_states,
                    update_compositor_size,
                ),
            );
    }
}