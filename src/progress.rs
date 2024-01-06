use bevy::{prelude::*, sprite::MaterialMesh2dBundle};

use crate::{player::{Player, Inventory}, crafting::Buildable, tile_world::{GameObject, TileType, create_bundle_for_tile, TileAssets, ObjectType}};

#[derive(Component)]
pub struct DestroyProgress {
    pub target: Entity,
    pub others: Vec<Entity>,
    pub get_inv: Inventory,
    pub start_time: f32,
    pub time_to_destroy: f32,
}

#[derive(Component)]
pub struct BuildProgress {
    pub others: Vec<Entity>,
    pub price_inv: Inventory,
    pub start_time: f32,
    pub time_to_build: f32,
    pub buildable: Buildable,
}

pub struct ProgressPlugin;

#[derive(Resource)]
pub struct ProgressStuff {
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
    bg_material: Handle<ColorMaterial>,
}

impl Plugin for ProgressPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
        app.add_systems(Update, update_destroy);
        app.add_systems(Update, update_build);
    }
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.insert_resource(ProgressStuff {
        mesh: meshes.add(shape::Quad::new(Vec2::new(1.0, 1.0)).into()),
        material: materials.add(Color::rgb(1.0, 0.3, 0.3).into()),
        bg_material: materials.add(Color::rgb(0.0, 0.0, 0.0).into()),
    });
}

pub fn start_destroy_progress(
    mut progress: DestroyProgress,
    commands: &mut Commands,
    progress_stuff: Res<ProgressStuff>,
    pos: Vec2,
) {
    let bg = commands.spawn(MaterialMesh2dBundle {
        material: progress_stuff.bg_material.clone(),
        mesh: progress_stuff.mesh.clone().into(),
        transform: Transform::from_translation(Vec3::new(pos.x, pos.y, 9.0)).with_scale(Vec3::new(0.55, 0.1, 1.0)),
        ..Default::default()
    });
    progress.others.push(bg.id());
    commands.spawn((
        MaterialMesh2dBundle {
            material: progress_stuff.material.clone(),
            mesh: progress_stuff.mesh.clone().into(),
            transform: Transform::from_translation(Vec3::new(pos.x, pos.y, 10.0)).with_scale(Vec3::new(0.0, 0.03, 1.0)),
            ..Default::default()
        }, progress
    ));
}

pub fn start_build_progress(
    mut progress: BuildProgress,
    commands: &mut Commands,
    progress_stuff: Res<ProgressStuff>,
    pos: Vec2,
) {
    let bg = commands.spawn(MaterialMesh2dBundle {
        material: progress_stuff.bg_material.clone(),
        mesh: progress_stuff.mesh.clone().into(),
        transform: Transform::from_translation(Vec3::new(pos.x, pos.y, 9.0)).with_scale(Vec3::new(0.55, 0.1, 1.0)),
        ..Default::default()
    });
    progress.others.push(bg.id());
    commands.spawn((
        MaterialMesh2dBundle {
            material: progress_stuff.material.clone(),
            mesh: progress_stuff.mesh.clone().into(),
            transform: Transform::from_translation(Vec3::new(pos.x, pos.y, 10.0)).with_scale(Vec3::new(0.0, 0.03, 1.0)),
            ..Default::default()
        }, progress
    ));
}

fn update_destroy(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut DestroyProgress)>,
    mut players: Query<&mut Player>,
    input: Res<Input<KeyCode>>,
) {

    // cancel all when player moves
    if input.any_pressed(vec![KeyCode::W, KeyCode::A, KeyCode::S, KeyCode::D]) {
        for (entity, _, progress) in &mut query {
            for other in progress.others.iter() {
                commands.entity(*other).despawn();
            }
            commands.entity(entity).despawn();
        }
    }

    // update progress bar
    for (entity, mut transform, progress) in &mut query {
        let progress_time = time.elapsed_seconds() - progress.start_time;
        let progress_percent = progress_time / progress.time_to_destroy;
        transform.scale.x = 0.5 * progress_percent;
        if progress_percent >= 1.0 {
            // completed!

            let mut player = players.iter_mut().next().expect("no player found");
            player.inventory = player.inventory + progress.get_inv;

            for other in progress.others.iter() {
                commands.entity(*other).despawn();
            }
            commands.entity(progress.target).despawn();
            commands.entity(entity).despawn();
        }
    }
}

fn update_build(
    time: Res<Time>,
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut BuildProgress)>,
    input: Res<Input<KeyCode>>,
    tile_assets: Res<TileAssets>
) {

    // cancel all when player moves
    if input.any_pressed(vec![KeyCode::W, KeyCode::A, KeyCode::S, KeyCode::D]) {
        for (entity, _, progress) in &mut query {
            for other in progress.others.iter() {
                commands.entity(*other).despawn();
            }
            commands.entity(entity).despawn();
        }
    }

    // update progress bar
    for (entity, mut transform, progress) in &mut query {
        let progress_time = time.elapsed_seconds() - progress.start_time;
        let progress_percent = progress_time / progress.time_to_build;
        transform.scale.x = 0.5 * progress_percent;
        if progress_percent >= 1.0 {
            // completed!

            for other in progress.others.iter() {
                commands.entity(*other).despawn();
            }
            commands.entity(entity).despawn();

            let tile_id = match progress.buildable {
                Buildable::Campfire => GameObject::from(ObjectType::Campfire).tile_id,
                Buildable::House => todo!("implement house spawn"),
                Buildable::Ship => todo!("implement ship spawn"),
            };
            commands.spawn((
                create_bundle_for_tile(transform.translation.x as usize, transform.translation.y as usize, tile_id, 2.0, &*tile_assets),
                GameObject { tile_id: tile_id },
            ));
        }
    }
}