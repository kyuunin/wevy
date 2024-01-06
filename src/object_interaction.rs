use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::prelude::*;
use bevy::sprite::collide_aabb;

use crate::player::Player;
use crate::tile_world::{GameObject, ObjectType};

pub struct ObjectInteractionPlugin;

impl Plugin for ObjectInteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
        app.add_systems(Update, update);
    }
}

#[derive(Component, Debug)]
struct InteractText;

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
}

fn update(
    mut players: Query<(&Player, &Transform)>,
    mut objects: Query<(&GameObject, &Transform)>,
    mut texts: Query<(&mut InteractText, &mut Text, &mut Visibility)>,
    mut key_evr: EventReader<KeyboardInput>,
) {
    let player_transform = players.iter().next().expect("no player found").1;

    let object = objects.iter_mut().find(|(_, tile_transform)| collide_aabb::collide(
            tile_transform.translation,
            Vec2::new(1.0, 1.0),
            player_transform.translation,
            player_transform.scale.truncate() * 32.0
    ).is_some());

    let mut text = texts.iter_mut().next().expect("no text found");

    match object {
        Some((object, _)) => {
            let desc: String = match object.get_type() {
                Some(ObjectType::Tree) => "cut down tree".to_string(),
                Some(ObjectType::Ship) => "loot ship".to_string(),
                None => format!("[TEXT MISSING TO PICK UP {:?}", object),
            };
            text.1.sections[0].value = format!("[E] {} [unimplemented]", desc);
            *text.2 = Visibility::Visible;
        },
        None => {
            *text.2 = Visibility::Hidden;
        }
    }

    if let Some((object, _)) = object { 
        if key_evr.read().any(|ev| ev.state == ButtonState::Pressed && ev.key_code == Some(KeyCode::E)) {
            let mut inventory = players.iter_mut().next().expect("no player found").0.inventory;
            match object.get_type() {
                Some(ObjectType::Tree) => {
                    inventory.wood += 10;
                    println!("You have {} wood", inventory.wood);
                },
                Some(ObjectType::Ship) => {
                    inventory.weapons += 10;
                    println!("You have {} weapons", inventory.weapons);
                },
                None => {
                    error!("unimplemented: pick up {:?}", object);
                }
            }
        }
    }
}
