use bevy::app::{App, Startup, Update};
use bevy::ecs::component::Component;
use bevy::ecs::system::Commands;
use bevy::math::{Vec2, Rect};
use bevy::render::view::Visibility;
use bevy::sprite::collide_aabb;
use bevy::text::{TextAlignment, TextStyle, Text};
use bevy::ui::{Style, PositionType, Val};
use bevy::ui::node_bundles::TextBundle;
use bevy::{app::Plugin, ecs::system::Query, transform::components::Transform};

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
    // commands.spawn(
    //     Text2dBundle {
    //         ..Default::default()
    //     }
    // );
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
    players: Query<(&Player, &Transform)>,
    mut objects: Query<(&GameObject, &Transform)>,
    mut texts: Query<(&mut InteractText, &mut Text, &mut Visibility)>,
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
}
