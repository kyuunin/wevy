use bevy::prelude::*;

fn hello_world_system() {
    println!("hello bevy!");
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Update, hello_world_system)
        .run();
}
