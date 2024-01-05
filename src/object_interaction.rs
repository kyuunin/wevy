use bevy::app::{Update, App};
use bevy::sprite::collide_aabb;
use bevy::{app::Plugin, ecs::system::Query, transform::components::Transform};

use crate::player::Player;
use crate::tile_world::TileEntity;

pub struct ObjectInteractionPlugin;

impl Plugin for ObjectInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update);
    }
}

fn update(
    players: Query<(&Player, &Transform)>,
    mut objects: Query<(&TileEntity, &Transform)>,
) {
    for (object, tile_transform) in objects.iter_mut() {
        for (_, player_transform) in &mut players.iter() {

            // do AABB collision detection
            if let Some(collision) = collide_aabb::collide(tile_transform.translation, tile_transform.scale.truncate(),
                player_transform.translation, player_transform.scale.truncate()) {
                
                println!("collision: {:?} with tile {:?}", collision, object);
            }
        }
    }
}
