// disable windows console window in release builds: https://bevy-cheatbook.github.io/platforms/windows.html#disabling-the-windows-console
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod game_object;
mod game_tile;
mod player;

use bevy::{log::LogPlugin, prelude::*};
use crafting::CraftingPlugin;
use object_interaction::ObjectInteractionPlugin;
use progress::ProgressPlugin;

use crate::player::PlayerPlugin;
use crate::tile_world::TileWorldPlugin;

mod crafting;
mod multi_vec;
mod object_interaction;
mod progress;
mod tile_world;
mod wave_function_collapse_generator;

#[cfg(feature = "inspect")]
use bevy_inspector_egui::quick::WorldInspectorPlugin;

fn main() {
    #[cfg(debug_assertions)]
    println!("Running in debug mode");

    // https://bevy-cheatbook.github.io/fundamentals/log.html
    #[cfg(debug_assertions)]
    let log_level = bevy::log::Level::DEBUG;
    #[cfg(not(debug_assertions))]
    let log_level = bevy::log::Level::INFO;

    println!("Use Log level: {}", log_level);

    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(ImagePlugin::default_nearest()) // prevents blurry sprites
            .set(LogPlugin {
                level: log_level,
                ..default()
            }),
    );

    #[cfg(feature = "inspect")]
    app.add_plugins(WorldInspectorPlugin::new());

    app.add_plugins(PlayerPlugin)
        .add_plugins(TileWorldPlugin)
        .add_plugins(ProgressPlugin)
        .add_plugins(CraftingPlugin)
        .add_plugins(ObjectInteractionPlugin);

    app.run();
}
