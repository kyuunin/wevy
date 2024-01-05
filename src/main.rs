//! Renders an animated sprite by loading all animation frames from a single image (a sprite sheet)
//! into a texture atlas, and changing the displayed image periodically.
mod player;


use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;

use crate::player::PlayerPlugin;
use crate::tile_world::TileWorldPlugin;

mod tile_world;
mod multi_vec;
mod wave_function_collapse_generator;

fn main() {
    
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // prevents blurry sprites
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins(PlayerPlugin)
        .add_plugins(TileWorldPlugin)
        .run();
}

