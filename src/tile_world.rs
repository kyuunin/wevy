use std::{sync::{Mutex, mpsc}, collections::{HashMap, HashSet}, cmp::{min, max}, ops::Not};

use bevy::{prelude::*, asset::LoadState, sprite::collide_aabb};
use bevy_common_assets::json::JsonAssetPlugin;
use rand::prelude::*;
use serde::Deserialize;

use crate::{
    multi_vec::MultiVec,
    game_tile::{
        MapData,
        GameTile,
        TileType,
    },
    game_object::GameObject, player::Player, wave_function_collapse_generator::WaveFunctionCollapseGenerator,
};

pub struct TileWorldPlugin;
impl Plugin for TileWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<PyxelFile>::new(&["json"])); // register .json extension (example advises for *."map.json")
        app.add_systems(PreStartup, pre_setup);
        app.add_systems(PreUpdate, generate_on_load_complete);
        app.register_type::<GameObject>();
        app.register_type::<GameTile>();
        app.init_resource::<MapData>();
    }
    fn name(&self) -> &str { "TileWorldPlugin" }
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
pub struct TileAssets {
    pyxel_file: Handle<PyxelFile>,
    tileset: Handle<Image>,
    generation_started: bool,
    has_moved_player: bool,
    rx: Option<Mutex<mpsc::Receiver<(usize, usize, i32)>>>,
    texture_atlas: Handle<TextureAtlas>,
    spawn_entities_for_base_tile: HashMap<i32, HashSet<i32>>,
}

fn single_collision(pos: Vec3, player_transform: &Transform) -> bool{
    collide_aabb::collide(
        pos,
        Vec2::new(0.5, 0.5),
        player_transform.translation,
        player_transform.scale.truncate() * 32.0
    ).is_some()
}

pub fn check_collision(
    map_data: &MapData,
    tiles: &Query<&GameTile>,
    player_transform: &Transform,
) -> bool {

    let player_pos = player_transform.translation;
    
    for x_offset in -1..=1 {
        for y_offset in -1..=1 {
            let Some((x, y, tile)) = map_data.get_tile_at_pos(
                Vec2::new(player_pos.x + x_offset as f32, player_pos.y + y_offset as f32), &tiles) 
             else {
                warn!("Couldn't get tile {} {}", (player_pos.x + x_offset as f32).round(), (player_pos.y + y_offset as f32).round());
                return true;
            };
            let x = x as f32;
            let y = y  as f32;
            
            if tile.top_left_type().map(TileType::can_enter).unwrap_or(true).not() && single_collision(
                Vec3{x: x-0.25, y: y+0.25, z: 1.},
                player_transform,
            ) {
                return true;
            }
            
                        
            if tile.top_right_type().map(TileType::can_enter).unwrap_or(true).not() && single_collision(
                Vec3{x: x+0.25, y: y+0.25, z: 1.},
                player_transform,
            ) {
                return true;
            }
            
                        
            if tile.bottom_left_type().map(TileType::can_enter).unwrap_or(true).not() && single_collision(
                Vec3{x: x-0.25, y: y-0.25, z: 1.},
                player_transform,
            ) {
                return true;
            }
            
                        
            if tile.bottom_right_type().map(TileType::can_enter).unwrap_or(true).not() && single_collision(
                Vec3{x: x+0.25, y: y-0.25, z: 1.},
                player_transform,
            ) {
                return true;
            }
        }
    };
    false
}

fn pre_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let pyxel_handle: Handle<PyxelFile> = asset_server.load("json/Map.json");
    let tileset_handle: Handle<Image> = asset_server.load("textures/Map.png");

    commands.insert_resource(TileAssets {
        pyxel_file: pyxel_handle,
        tileset: tileset_handle,
        generation_started: false,
        texture_atlas: default(),
        has_moved_player: false,
        rx: None,
        spawn_entities_for_base_tile: HashMap::new(),
    });
}

pub fn create_bundle_for_tile(x: usize, y: usize, tile_id: i32, z: f32,
    tile_assets: &TileAssets
) -> SpriteSheetBundle {
    SpriteSheetBundle {
        texture_atlas: tile_assets.texture_atlas.clone(),
        sprite: TextureAtlasSprite::new(tile_id as usize),
        transform:
            Transform::from_translation(Vec3::new(x as f32, y as f32, z))
            .with_scale(Vec3::new(1.0 / 32.0, 1.0 / 32.0, 1.0)),
        ..Default::default()
    }
}

fn generate_on_load_complete(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut tile_assets: ResMut<TileAssets>,
    mut map_data: ResMut<MapData>,
    pyxel_file_assets: Res<Assets<PyxelFile>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    mut player_query: Query<(&Player, &mut Transform)>,
) {
    if !tile_assets.generation_started {
        let pyxel_load_state = asset_server.get_load_state(tile_assets.pyxel_file.id())
            .expect("asset_server.get_load_state returns Option<LoadState>, should be Some");

        if pyxel_load_state == LoadState::Loaded {
            println!("pyxel file loaded!!!!!!11");

            let pyxel_file = pyxel_file_assets.get(&tile_assets.pyxel_file).expect(
                "pyxel json file should be loaded since we checked that LoadState::Loaded"
            );
            start_generation(pyxel_file, &mut *tile_assets, &mut *texture_atlases, &mut *map_data);

            tile_assets.generation_started = true;
        }

    }

    spawn_generated(&mut commands, &mut *tile_assets, &mut player_query.single_mut().1, &mut *map_data);
}

fn start_generation(
    pyxel_file: &PyxelFile,
    tile_assets: &mut TileAssets,
    texture_atlases: &mut Assets<TextureAtlas>,
    map_data: &mut MapData,
) {
    for layer in pyxel_file.layers.iter() {
        println!("layer {:?}: {:?}", layer.number, layer.name);
    }

    let base_layer = pyxel_file.layers.iter().find(|layer| layer.number == 2).unwrap();
    let entity_layer = pyxel_file.layers.iter().find(|layer| layer.number == 1).unwrap();

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

    // for each entity tile type, spawn on base layer tiles
    tile_assets.spawn_entities_for_base_tile = HashMap::<i32, HashSet<i32>>::new();
    for entity_tile in entity_layer.tiles.iter().filter(|tile| tile.tile != -1) {
        let base_tile = base_layer.tiles.iter()
            .find(|tile| tile.x == entity_tile.x && tile.y == entity_tile.y).unwrap();
        tile_assets.spawn_entities_for_base_tile.entry(base_tile.tile).or_insert(default()).insert(entity_tile.tile);
    }

    let (tx, rx) = std::sync::mpsc::channel();
    tile_assets.rx = Some(Mutex::new(rx));

    let map_size = 64;

    std::thread::spawn(move || {
        
        let generator = WaveFunctionCollapseGenerator::new(
            tiles,
            map_size,
            2,
            666
        );

        #[cfg(debug_assertions)]
        let mut map = MultiVec::new(-1, map_size, map_size);

        for (x, y, tile_id) in generator {
            tx.send((x, y, tile_id)).unwrap();

            #[cfg(debug_assertions)]
            {
                *map.get_mut(x, y).unwrap() = tile_id;
            }
        }
        
        #[cfg(debug_assertions)]
        for y in 0..map.h {
            for x in 0..map.w {
                print!("{:2}", map.get(x, y).unwrap());
            }
            println!();
        }
    });

    let texture_atlas = TextureAtlas::from_grid(
        tile_assets.tileset.clone(),
        Vec2::new(pyxel_file.tilewidth as f32, pyxel_file.tileheight as f32),
        8, 8, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);
    tile_assets.texture_atlas = texture_atlas_handle;

    *map_data = MapData { 0: MultiVec::new(None, map_size, map_size) };
}

fn spawn_generated(
    commands: &mut Commands,
    tile_assets: &mut TileAssets,
    player_transform: &mut Transform,
    map_data: &mut MapData,
) {
    if tile_assets.rx.is_none() {
        return;
    }

    let rx = tile_assets.rx.as_ref().unwrap().lock().unwrap();
    let max_tiles_per_frame = 32;
    for _ in 0..max_tiles_per_frame {
        let next = rx.try_recv();
        if next.is_err() { break; }
        let (x, y, tile_id) = next.unwrap();
        
        debug!("rx received: ({},{}) = {}", x, y, tile_id);
        
        if tile_id == -1 {
            warn!("rx received tile_id == -1");
            return;
        }

        let base_entity = commands.spawn((
            create_bundle_for_tile(x, y, tile_id, -1.0, &*tile_assets),
            GameTile { tile_id },
            Name::new(format!("Tile {tile_id} ({x},{y})")),
        )).id();

        let map_data_cell = map_data.0.get_mut(x, y).expect("accessing map data failed");
        assert!(map_data_cell.is_none(), "map data cell should be None");
        *map_data_cell = Some(base_entity);

        // Generate entities on correct base tiles
        let spawn_rate = 0.2 as f32;

        if let Some(entities) = tile_assets.spawn_entities_for_base_tile.get(&tile_id) {
            if rand::random::<f32>() < spawn_rate {
                let entity = entities.iter().choose(&mut rand::thread_rng()).expect("should have at least one entity");
                commands.spawn((
                    create_bundle_for_tile(x, y, *entity, -0.5, &*tile_assets),
                    GameObject { tile_id: *entity },
                    Name::new(format!("Object {entity} ({x},{y})")),
                ));
            }
        }

        let player_spawn_tile = 9;
        if !tile_assets.has_moved_player && tile_id == player_spawn_tile {
            player_transform.translation = Vec3::new(x as f32, y as f32, player_transform.translation.z);
            tile_assets.has_moved_player = true;
        }
    }
}
