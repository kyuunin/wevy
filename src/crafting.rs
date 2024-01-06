use bevy::{prelude::*, input::{keyboard::KeyboardInput, ButtonState}};

use crate::{
    player::{Inventory, Player},
    tile_world::{create_bundle_for_tile, ObjectType, GameObject, TileAssets},
    progress::{self, BuildProgress},
    game_tile::{GameTile, TileType, MapData},
};


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
    mut commands: Commands,
    mut recipe_texts: Query<(&mut RecipeText, &mut Text)>,
    mut crafting_state: ResMut<CraftingState>,
    mut key_evr: EventReader<KeyboardInput>,
    mut players: Query<(&mut Player, &Transform)>,
    map_data: Res<MapData>,
    tiles: Query<&GameTile>,
    tile_assets: Res<TileAssets>,
    progress_stuff: Res<progress::ProgressStuff>,
    time: Res<Time>,
) {
    let mut recipe_text = recipe_texts.iter_mut().next().expect("no recipe text found");
    recipe_text.1.sections[0].value = format!("[R] to build {} ({})\n[Q] next recipe", crafting_name(crafting_state.recipe), crafting_price(crafting_state.recipe).to_string());

    let key_evr = key_evr.read().collect::<Vec<_>>();

    // Cheats: Number keys add resources
    #[cfg(debug_assertions)]
    {
        let (mut player, _) = players.iter_mut().next().expect("no player found");
        if key_evr.iter().any(|ev| ev.state == ButtonState::Pressed && ev.key_code == Some(KeyCode::Key1)) {
            player.inventory.wood += 10;
        }
        if key_evr.iter().any(|ev| ev.state == ButtonState::Pressed && ev.key_code == Some(KeyCode::Key2)) {
            player.inventory.stone += 10;
        }
        if key_evr.iter().any(|ev| ev.state == ButtonState::Pressed && ev.key_code == Some(KeyCode::Key3)) {
            player.inventory.weapons += 10;
        }
    }

    // [Q] to change recipe
    if key_evr.iter().any(|ev| ev.state == ButtonState::Pressed && ev.key_code == Some(KeyCode::Q)) {
        crafting_state.recipe = match crafting_state.recipe {
            Buildable::Ship => Buildable::Campfire,
            Buildable::Campfire => Buildable::House,
            Buildable::House => Buildable::Ship,
        };
    }

    // [R] to build
    if key_evr.iter().any(|ev| ev.state == ButtonState::Pressed && ev.key_code == Some(KeyCode::R)) {
        let (mut player, transform) = players.iter_mut().next().expect("no player found");
        let price = crafting_price(crafting_state.recipe);
        let Inventory { wood: wood_price, stone: stone_price, weapons: weapons_price } = price;
        if player.inventory.wood >= wood_price && player.inventory.stone >= stone_price && player.inventory.weapons >= weapons_price {
            let (x, y, tile) = map_data.get_tile_at_pos(transform.translation.truncate(), &tiles).expect("no tile found");
            let has_water = tile.bottom_left_type() == Some(TileType::Water)
                || tile.bottom_right_type() == Some(TileType::Water)
                || tile.top_left_type() == Some(TileType::Water)
                || tile.top_right_type() == Some(TileType::Water);
            let has_land = tile.bottom_left_type() == Some(TileType::Field)
                || tile.bottom_right_type() == Some(TileType::Field)
                || tile.top_left_type() == Some(TileType::Field)
                || tile.top_right_type() == Some(TileType::Field);

            match crafting_state.recipe {
                Buildable::Ship => {
                    todo!("implement ship")
                },
                Buildable::Campfire => {
                    if has_land {
                        info!("You can now cook meat on the campfire");
                        progress::start_build_progress(BuildProgress {
                            others: vec![],
                            price_inv: price,
                            start_time: time.elapsed_seconds(),
                            time_to_build: 5.0,
                            buildable: Buildable::Campfire,
                        }, &mut commands, progress_stuff, Vec2::new(x as f32, y as f32));
                    } else {
                        info!("You can only build a campfire on land");
                    }
                },
                Buildable::House => {
                    todo!("implement house");
                },
            };
        } else {
            info!("Not enough resources to build {}", crafting_name(crafting_state.recipe));
        }
    }
}
