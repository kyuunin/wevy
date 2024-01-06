use bevy::{prelude::*, asset::LoadState};
use bevy_common_assets::json::JsonAssetPlugin;
use serde::Deserialize;
use std::{cmp::{min, max}, collections::{HashMap, HashSet}, ops::Deref};
use rand::prelude::*;

use crate::multi_vec::MultiVec;
use crate::player::Player;

pub struct TileWorldPlugin;
impl Plugin for TileWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<PyxelFile>::new(&["json"])); // register .json extension (example advises for *."map.json")
        app.add_systems(PreStartup, pre_setup);
        app.add_systems(Update, test);
        app.add_systems(PreUpdate, generate_on_load_complete);
        app.register_type::<GameObject>();
        app.register_type::<GameTile>();
        app.init_resource::<MapData>();
    }
    fn name(&self) -> &str { "TileWorldPlugin" }
}

#[derive(Default, Resource)]
pub struct MapData(MultiVec<Option<Entity>>);

#[derive(Debug, PartialEq, Eq)]
pub enum TileType {
    Water, Field, Mountain,
}

#[derive(Debug)]
pub enum ObjectType {
    Tree, Ship, Stone, Campfire
}

impl From<ObjectType> for GameObject {
    fn from(object_type: ObjectType) -> Self {
        use ObjectType::*;
        match object_type {
            Tree => GameObject { tile_id: 12 },
            Ship => GameObject { tile_id: 13 },
            Stone => GameObject { tile_id: 27 },
            Campfire => GameObject { tile_id: 29 },
        }
    }
}

#[derive(Component, Debug, Reflect)]
pub struct GameObject {
    tile_id: i32,
}

#[derive(Component, Debug, Reflect, Clone, Copy)]
pub struct GameTile {
    tile_id: i32,
}

impl GameObject {
    pub fn get_type(&self) -> Option<ObjectType> {
        use ObjectType::*;
        match self.tile_id {
            12 => Some(Tree),
            13 => Some(Ship),
            27 => Some(Stone),
            29 => Some(Campfire),
            _  => None,
        }
    }
}

impl GameTile {
    pub fn top_left_type(&self) -> Option<TileType> {
        use TileType::*;
        match self.tile_id {
             0| 1| 2| 3| 8|11|15                   => Some(Water),
             4| 5| 6| 7| 9|10|14|16|17|18|19|24|31 => Some(Field),
            20|21|22|23|25|26|30                   => Some(Mountain),
            _                                      => None,
        }
    }
    
    pub fn top_right_type(&self) -> Option<TileType> {
        use TileType::*;
        match self.tile_id {
             0| 1| 2| 5|10|11|14                   => Some(Water),
             3| 4| 6| 7| 8| 9|15|16|17|18|21|26|30 => Some(Field),
            19|20|22|23|24|25|31                   => Some(Mountain),
            _                                      => None,
        }
    }
    pub fn bottom_left_type(&self) -> Option<TileType> {
        use TileType::*;
        match self.tile_id {
             0| 3| 4| 5| 7| 8|11                   => Some(Water),
             1| 2| 6| 9|10|14|15|16|19|20|21|23|24 => Some(Field),
            17|18|22|25|26|30|31                   => Some(Mountain),
            _                                      => None,
        }
    }
    
    pub fn bottom_right_type(&self) -> Option<TileType> {
        use TileType::*;
        match self.tile_id {
             2| 3| 4| 5| 6|10|11                   => Some(Water),
             0| 1| 2| 8| 9|14|15|18|19|20|21|22|26 => Some(Field),
            16|17|23|24|25|30|31                   => Some(Mountain),
            _                                      => None,
        }
    }
}

#[derive(Deserialize, Asset, TypePath)]
struct PyxelFile {
    // tileswide: i32, // number of tiles in tilemap in x direction, e.g. 10
    // tileshigh: i32, // number of tiles in tilemap in y direction, e.g. 12
    tilewidth: i32,  // width  of single tile in pixels, e.g. 32
    tileheight: i32, // height of single tile in pixels, e.g. 32
    layers: Vec<PyxelLayer>,
}

#[derive(Deserialize)]
struct PyxelLayer {
    name: String,
    number: i32,
    tiles: Vec<PyxelTile>,
}

#[derive(Deserialize)]
struct PyxelTile {
    x: i32,
    y: i32,
    tile: i32, // index of tile in tileset, or -1 for empty / custom tile
    // index: i32, // not sure, maybe y * width + x?
    // #[serde(rename = "flipX")]
    // flip_x: bool,
    // rot: i32, // 0, 1, 2, 3
}

#[derive(Resource)]
struct TileAssets {
    pyxel_file: Handle<PyxelFile>,
    tileset: Handle<Image>,
    has_generated: bool,
}

pub fn get_tile_at_pos(pos: Vec2, map_data: Res<MapData>, tiles: Query<&GameTile>) -> Option<(usize, usize, GameTile)> {
    let pos = pos.round();
    if pos.x < -0.5 {return None}
    let x = pos.x as usize;
    if pos.y < -0.5 {return None}
    let y = pos.y as usize;
    let entity = map_data.0.get(x, y)?.as_ref()?;
    Some((x, y, *tiles.get_component::<GameTile>(*entity).ok()?))
}

fn test(
    map_data: Res<MapData>,
    tiles: Query<&GameTile>,
    mut players: Query<(&mut Player, &Transform)>,
) {

    let player_pos = players.iter().next().expect("no player found").1.translation;
    let Some((x, y, tile)) = get_tile_at_pos(player_pos.truncate(), map_data, tiles) else {
        warn!("Couldn't get tile");
        return;
    };
    println!("{x},{y} = {:?}",tile.top_left_type());
    println!("{x},{y} = {:?}",tile.top_right_type());
    println!("{x},{y} = {:?}",tile.bottom_left_type());
    println!("{x},{y} = {:?}",tile.bottom_right_type());
}

fn pre_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let pyxel_handle: Handle<PyxelFile> = asset_server.load("json/Map.json");
    let tileset_handle: Handle<Image> = asset_server.load("textures/Map.png");

    commands.insert_resource(TileAssets {
        pyxel_file: pyxel_handle,
        tileset: tileset_handle,
        has_generated: false,
    });
}

fn generate_on_load_complete(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut tile_assets: ResMut<TileAssets>,
    mut map_data: ResMut<MapData>,
    pyxel_file_assets: Res<Assets<PyxelFile>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    if tile_assets.has_generated {
        return;
    }

    match asset_server.get_load_state(tile_assets.pyxel_file.id()).expect(
        "asset_server.get_load_state returns Option<LoadState>, should be Some") {
        LoadState::Loaded => {
            println!("pyxel file loaded!!!!!!11");
            let pyxel_file = pyxel_file_assets.get(&tile_assets.pyxel_file).expect(
                "pyxel json file should be loaded since we checked that LoadState::Loaded"
            );

            for layer in pyxel_file.layers.iter() {
                println!("layer {:?}: {:?}", layer.number, layer.name);
            }

            let base_layer = pyxel_file.layers.iter().find(|layer| layer.number == 1).unwrap();
            let entity_layer = pyxel_file.layers.iter().find(|layer| layer.number == 0).unwrap();

            let min_tile = base_layer.tiles.iter()
                .filter(|tile| tile.tile != -1)
                .map(|tile| (tile.x, tile.y))
                .fold((i32::MAX, i32::MAX), |(min_x, min_y), (x, y)| (min(min_x, x), min(min_y, y)));
            let max_tile = base_layer.tiles.iter()
                .filter(|tile| tile.tile != -1)
                .map(|tile| (tile.x, tile.y))
                .fold((i32::MIN, i32::MIN), |(max_x, max_y), (x, y)| (max(max_x, x), max(max_y, y)));
            println!("min_tile: {:?}, max_tile: {:?}", min_tile, max_tile);

            let mut tiles = MultiVec::new(-1, (max_tile.0 - min_tile.0 + 1) as usize, (max_tile.1 - min_tile.1 + 1) as usize);
            for tile in base_layer.tiles.iter() {
                if tile.tile != -1 {
                    let x = (tile.x - min_tile.0) as usize;
                    let y = (tile.y - min_tile.1) as usize;
                    let flipped_y = (max_tile.1 - min_tile.1) as usize - y;
                    *(tiles.get_mut(x, flipped_y).unwrap()) = tile.tile;
                }
            }

            // TODO: call let map_data = david(tiles)
            let map = tiles.clone();

            for y in min_tile.1..=max_tile.1 {
                for x in min_tile.0..=max_tile.0 {
                    let tile = tiles.get((x - min_tile.0) as usize, (y - min_tile.1) as usize).unwrap();
                    print!("{:2} ", tile);
                }
                println!();
            }

            let texture_atlas = TextureAtlas::from_grid(
                tile_assets.tileset.clone(),
                Vec2::new(pyxel_file.tilewidth as f32, pyxel_file.tileheight as f32),
                8, 8, None, None);
            let texture_atlas_handle = texture_atlases.add(texture_atlas);

            // let scale = 3.0;
            // let tile_scale_x = scale * pyxel_file.tilewidth as f32;
            // let tile_scale_y = scale * pyxel_file.tileheight as f32;

            let tile_scale_x: f32 = 1.0;
            let tile_scale_y: f32 = 1.0;
            let scale: f32 = 1.0 / 32.0;
            
            *map_data.as_mut() = MapData(MultiVec::new(None, map.w, map.h));
            for y in 0..map.h {
                for x in 0..map.w {
                    let tile = map.get(x, y).unwrap();
                    if *tile != -1 {
                        let id = commands.spawn((
                            SpriteSheetBundle {
                                texture_atlas: texture_atlas_handle.clone(),
                                sprite: TextureAtlasSprite::new(*tile as usize),
                                transform: Transform::from_translation(Vec3::new(tile_scale_x * x as f32, tile_scale_y * y as f32, -1.0))
                                    .with_scale(Vec3::new(scale, scale, 1.0)),
                                ..Default::default()
                            },
                            GameTile { tile_id: *tile },
                            Name::new(format!("Tile {tile} ({x},{y})")),
                        )).id();
                        *map_data.as_mut().0.get_mut(x, y).expect("Storing map data failed") = Some(id);
                    }
                }
            }

            // for each entity tile type, spawn on base layer tiles
            let mut spawn_entities_for_base_tile = HashMap::<i32, HashSet<i32>>::new();
            for entity_tile in entity_layer.tiles.iter().filter(|tile| tile.tile != -1) {
                let base_tile = base_layer.tiles.iter()
                    .find(|tile| tile.x == entity_tile.x && tile.y == entity_tile.y).unwrap();
                spawn_entities_for_base_tile.entry(base_tile.tile).or_insert(default()).insert(entity_tile.tile);
            }

            // Generate entities on correct base tiles
            let spawn_rate = 0.2 as f32;
            for y in 0..map.h {
                for x in 0..map.w {
                    let base_tile = map.get(x, y).unwrap();
                    if let Some(entities) = spawn_entities_for_base_tile.get(base_tile)  {
                        if rand::random::<f32>() < spawn_rate {
                            let entity = entities.iter().choose(&mut rand::thread_rng()).expect("should have at least one entity");
                            commands.spawn((
                                SpriteSheetBundle {
                                    texture_atlas: texture_atlas_handle.clone(),
                                    sprite: TextureAtlasSprite::new(*entity as usize),
                                    transform:
                                        Transform::from_translation(Vec3::new(tile_scale_x * x as f32, tile_scale_y * y as f32, -0.5))
                                        .with_scale(Vec3::new(scale, scale, 1.0)),
                                  ..Default::default()
                                }, 
                                GameObject { tile_id: *entity },
                                Name::new(format!("Object {entity} ({x},{y})")),
                            ));
                        }
                    }
                }
            }

            tile_assets.has_generated = true;
        },
        _ => {
            println!("pyxel file not loaded yet");
        }
    }
}
