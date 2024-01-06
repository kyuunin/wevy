use bevy::prelude::*;

use crate::player::Inventory;


pub struct CraftingPlugin;

impl Plugin for CraftingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, startup);
        app.add_systems(Update, update);
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Buildable {
    Ship,
    Campfire,
    House,
}

pub fn crafting_price(buildable: Buildable) -> Inventory {
    match buildable {
        Buildable::Ship => Inventory {
            wood: 100,
            stone: 50,
            weapons: 0,
        },
        Buildable::Campfire => Inventory {
            wood: 30,
            stone: 0,
            weapons: 0,
        },
        Buildable::House => Inventory {
            wood: 50,
            stone: 20,
            weapons: 0,
        },
    }
}

pub fn crafting_name (buildable: Buildable) -> &'static str {
    match buildable {
        Buildable::Ship => "Ship",
        Buildable::Campfire => "Campfire",
        Buildable::House => "House",
    }
}

#[derive(Component)]
pub struct RecipeText;

#[derive(Resource)]
pub struct CraftingState {
    recipe: Buildable,
}

fn startup(
    mut commands: Commands,
) {
    commands.insert_resource(CraftingState { recipe: Buildable::Ship });
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
            top: Val::Px(100.0),
            left: Val::Px(20.0),
            ..Default::default()
        }),
        RecipeText {},
    ));
}

fn update(
    mut recipe_texts: Query<(&mut RecipeText, &mut Text)>,
    mut crafting_state: Res<CraftingState>,
) {
    let mut recipe_text = recipe_texts.iter_mut().next().expect("no recipe text found");
    recipe_text.1.sections[0].value = format!("{}: {}", crafting_name(crafting_state.recipe), crafting_price(crafting_state.recipe));
}