use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::sprite::collide_aabb;

use crate::player::Player;
use crate::progress::{self, DestroyProgress};
use crate::tile_world::{GameObject, ObjectType};

pub struct ObjectInteractionPlugin;

impl Plugin for ObjectInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
        app.add_systems(Update, update);
        app.add_systems(Update, update_inventory_text);
    }
}

#[derive(Component, Debug)]
struct InteractText;

#[derive(Component, Debug)]
struct InventoryText;

fn startup(mut commands: Commands) {
    commands.spawn((
        TextBundle::from_section(
            "Test text",
            TextStyle {
                font_size: 20.0,
                ..Default::default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Percent(60.0),
            left: Val::Percent(30.0),
            ..Default::default()
        })
        .with_text_alignment(TextAlignment::Center),
        InteractText {},
    ));
    commands.spawn((
        TextBundle::from_section(
            "[empty inventory]",
            TextStyle {
                font_size: 20.0,
                ..Default::default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(20.0),
            left: Val::Px(20.0),
            ..Default::default()
        }),
        InventoryText {},
    ));
}

fn update(
    mut commands: Commands,
    mut players: Query<(&mut Player, &Transform)>,
    objects: Query<(Entity, &GameObject, &Transform)>,
    mut texts: Query<(&mut InteractText, &mut Text, &mut Visibility)>,
    mut key_evr: EventReader<KeyboardInput>,
    time: Res<Time>,
    progress_stuff: Res<progress::ProgressStuff>,
    running_progress: Query<&DestroyProgress>,
) {
    let (_, player_transform) = players.iter_mut().next().expect("no player found");
    let progress_running = running_progress.iter().next().is_some();

    let object = objects.iter().find(|(_, _, tile_transform)| collide_aabb::collide(
            tile_transform.translation,
            Vec2::new(1.0, 1.0),
            player_transform.translation,
            player_transform.scale.truncate() * 32.0
    ).is_some());

    let mut text = texts.iter_mut().next().expect("no text found");

    // show text to player
    let desc = object
        .and_then(|(_, object, _)| { match object.get_type() {
            Some(ObjectType::Tree) => Some("cut down tree"),
            Some(ObjectType::Ship) => Some("loot ship"),
            Some(ObjectType::Stone) => Some("mine stone"),
            Some(ObjectType::Campfire) => None,
            None => None,
        }})
        .map(|desc| format!("[E] {}", desc));
    if let Some(desc) = desc {
        text.1.sections[0].value = desc;
        *text.2 = Visibility::Visible;
    } else {
        *text.2 = Visibility::Hidden;
    }

    // handle key press: start destroy progress
    if let Some((entity, object, transform)) = object  {
        if !progress_running { Some(()) } else { None }
            .and(if key_evr.read().any(|ev| ev.state == ButtonState::Pressed && ev.key_code == Some(KeyCode::E)) { Some(()) } else { None })
            .and(Some(DestroyProgress {
                target: entity,
                others: vec![],
                get_inv: default(),
                start_time: time.elapsed_seconds(),
                time_to_destroy: 2.0,
            }))
            .and_then(|mut progress| {
                match object.get_type() {
                    Some(ObjectType::Tree) => progress.get_inv.wood += 10,
                    Some(ObjectType::Ship) => progress.get_inv.weapons += 10,
                    Some(ObjectType::Stone) => progress.get_inv.stone += 10,
                    Some(ObjectType::Campfire) => return None,
                    None => return None,
                }
                Some(progress)
            })
            .map(|progress| {
                progress::start_destroy_progress(progress, &mut commands, progress_stuff, transform.translation.truncate())
            });
    }
}

fn update_inventory_text(
    mut inv_text: Query<(&mut InventoryText, &mut Text)>,
    players: Query<&Player>) {

    let player = players.iter().next().expect("no player found");
    let mut inventory_text = inv_text.iter_mut().next().expect("no inventory text found");
    inventory_text.1.sections[0].value = format!("Wood: {}\nStone: {}\nWeapons: {}", player.inventory.wood, player.inventory.stone, player.inventory.weapons);
}