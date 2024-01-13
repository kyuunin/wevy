use std::{fmt::Formatter, fmt::Display};
use derive_more::{Add, Sub, AddAssign, SubAssign};
use std::ops::Not;
use bevy::prelude::*;
use crate::{
    game_tile::{
        MapData,
        GameTile,
    },
    tile_world::check_collision,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
  fn build(&self,  app: &mut App) {
    app.add_systems(Startup, setup)
       .add_systems(Update, animate_sprite)
       .add_systems(Update, keyboard_events);
  }
}

#[derive(Component)]
pub struct Player {
    pub inventory: Inventory,
    #[cfg(feature = "cheat")]
    pub ghost: bool,
}

impl Player {
    fn new() -> Self {
        #[cfg(feature = "cheat")]
        return Player { inventory: Inventory { wood: 0, stone: 0, weapons: 0 }, ghost: false, };
        #[cfg(not(feature = "cheat"))]
        return Player { inventory: Inventory { wood: 0, stone: 0, weapons: 0 },};
    }
    fn check_collision(
        &self, 
        map_data: &MapData,
        tiles: &Query<&GameTile>,
        player_transform: &Transform,
    ) -> bool {
        #[cfg(feature = "cheat")]
        return self.ghost.not() && check_collision(&map_data, &tiles, &player_transform);
        #[cfg(not(feature = "cheat"))]
        return check_collision(&map_data, &tiles, &player_transform);
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Add, Sub, AddAssign, SubAssign)]
pub struct Inventory {
    pub wood: i32,
    pub stone: i32,
    pub weapons: i32,
}
impl Display for Inventory {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        vec![if self.wood > 0 { Some(format!("{} wood", self.wood)) } else { None },
             if self.stone > 0 { Some(format!("{} stone", self.stone)) } else { None },
             if self.weapons > 0 { Some(format!("{} weapons", self.weapons)) } else { None }]
            .into_iter()
            .filter_map(|x| x)
            .collect::<Vec<String>>()
            .join(", ")
            .fmt(f)
    }
}

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
    mirrored: bool,
    walking: bool,
}

#[derive(Component, Deref, DerefMut)]
struct AnimationTimer(Timer);

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(
        &AnimationIndices,
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
    )>,
) {
    //trace!("very noisy");
    //debug!("helpful for debugging");
    //info!("helpful information that is worth printing by default");
    //warn!("something bad happened that isn't a failure, but thats worth calling out");
    //error!("something failed");
    for (indices, mut timer, mut sprite) in &mut query {
        timer.tick(time.delta());
        if indices.walking {
            if sprite.index < indices.first { sprite.index = indices.first } // immediately switch to walk anim when starting walking
            if timer.just_finished() {
                sprite.index = if sprite.index == indices.last {
                    indices.first
                } else {
                    sprite.index + 1
                };
            } 
        } else {
            sprite.index = indices.first - 1; // sprite sheet contains standing frame before walking frames
            timer.reset();
        }
        sprite.flip_x = indices.mirrored;
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("textures/gabe-idle-run.png");
    let texture_atlas =
        TextureAtlas::from_grid(texture_handle, Vec2::new(24.0, 24.0), 7, 1, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    // Use only the subset of sprites in the sheet that make up the run animation
    let animation_indices = AnimationIndices { first: 1, last: 6, mirrored: false, walking: false };

    let player_size = 0.6 / 32.0;
    let camera_scale = 0.007;

    let camera = commands.spawn(
    	Camera2dBundle{
    		transform: Transform::from_scale(Vec3::new(camera_scale / player_size, camera_scale / player_size, 1.0)),
    		..default()
	}).id();
    let player = commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite::new(animation_indices.first),
            transform: Transform::from_scale(Vec3::new(player_size, player_size, 1.0)),
            ..default()
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        Player::new(),
        Name::new("Player"),
        
    )).id();
    commands.entity(player).push_children(&[camera]);
    ()
}


fn keyboard_events(
    // mut key_evr: EventReader<KeyboardInput>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
    map_data: Res<MapData>,
    tiles: Query<&GameTile>,
    mut players: Query<(&mut Player, &mut AnimationIndices, &mut Transform)>,
) {
    let speed: f32 = 1.0;
    
    let (mut player, mut indices, mut player_transform) = players.iter_mut().next().expect("No player found");
    let mut transform = *player_transform;
    indices.walking = false;
    if input.pressed(KeyCode::W) {
        transform.translation.y += speed * time.delta_seconds();
        if player.check_collision(&map_data, &tiles, &transform){
            transform = *player_transform;
        } else {
            *player_transform = transform;
            indices.walking = true;
        }
    }
    if input.pressed(KeyCode::S) {
        transform.translation.y -= speed * time.delta_seconds();
        if player.check_collision(&map_data, &tiles, &transform){
            transform = *player_transform;
        } else {
            *player_transform = transform;
            indices.walking = true;
        }
    }
    if input.pressed(KeyCode::A) {
        transform.translation.x -= speed * time.delta_seconds();
        indices.mirrored = true;
        if player.check_collision(&map_data, &tiles, &transform){
            transform = *player_transform;
        } else {
            *player_transform = transform;
            indices.walking = true;
        }
    }
    if input.pressed(KeyCode::D) {
        transform.translation.x += speed * time.delta_seconds();
        indices.mirrored = false;
        if player.check_collision(&map_data, &tiles, &transform){
        } else {
            *player_transform = transform;
            indices.walking = true;
        }
    }
    #[cfg(feature = "cheat")]
    if input.pressed(KeyCode::X) {
        player.ghost = true;
    }
    #[cfg(feature = "cheat")]
    if input.pressed(KeyCode::Y) {
        player.ghost = false;
    }
    // use bevy::input::ButtonState;
    // for ev in key_evr.read() {
    //     match ev.state {
    //         ButtonState::Pressed => {
    //             error!("Key press: {:?} ({})", ev.key_code, ev.scan_code);
    //             match ev.key_code {
    //                 Some(KeyCode::A) => {
    //                     for (_, mut transform) in &mut players.iter_mut() {
    //                         transform.translation.x -= 10.0;
    //                     }
    //                 },
    //                 _ => {}
    //             }
    //         }
    //         ButtonState::Released => {
    //             error!("Key release: {:?} ({})", ev.key_code, ev.scan_code);
    //         }
    //     }
    // }
}
