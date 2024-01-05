use bevy::prelude::*;
use bevy::input::keyboard::KeyboardInput;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
  fn build(&self,  app: &mut App) {
    app.add_systems(Startup, setup)
       .add_systems(Update, animate_sprite)
       .add_systems(Update, keyboard_events);
  }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct AnimationIndices {
    first: usize,
    last: usize,
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
        if timer.just_finished() {
            sprite.index = if sprite.index == indices.last {
                indices.first
            } else {
                sprite.index + 1
            };
        }
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
    let animation_indices = AnimationIndices { first: 1, last: 6 };
    let camera = commands.spawn(Camera2dBundle::default(),).id();
    let player = commands.spawn((
        SpriteSheetBundle {
            texture_atlas: texture_atlas_handle,
            sprite: TextureAtlasSprite::new(animation_indices.first),
            transform: Transform::from_scale(Vec3::splat(3.0)),
            ..default()
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        Player,
        
    )).id();
    commands.entity(player).push_children(&[camera]);
    ()
}


fn keyboard_events(
    // mut key_evr: EventReader<KeyboardInput>,
    mut players: Query<(&AnimationTimer, &mut Transform)>,
    input: Res<Input<KeyCode>>,
    time: Res<Time>,
) {
    let speed = 100.0_f32;
    if input.pressed(KeyCode::W) {
        for (_, mut transform) in &mut players.iter_mut() {
            transform.translation.y += speed * time.delta_seconds();
        }
    }
    if input.pressed(KeyCode::S) {
        for (_, mut transform) in &mut players.iter_mut() {
            transform.translation.y -= speed * time.delta_seconds();
        }
    }
    if input.pressed(KeyCode::A) {
        for (_, mut transform) in &mut players.iter_mut() {
            transform.translation.x -= speed * time.delta_seconds();
        }
    }
    if input.pressed(KeyCode::D) {
        for (_, mut transform) in &mut players.iter_mut() {
            transform.translation.x += speed * time.delta_seconds();
        }
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
