use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;
use serde::Deserialize;
use std::cmp::{min, max};

use crate::multi_vec::MultiVec;


pub struct TileWorldPlugin;
impl Plugin for TileWorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<PyxelFile>::new(&["json"])); // register .json extension (example advises for *."map.json")
        app.add_systems(PreStartup, pre_setup);
        app.add_systems(Startup, setup);
    }
    fn name(&self) -> &str { "TileWorldPlugin" }
}

#[derive(Deserialize, Asset, TypePath)]
struct PyxelFile {
    tileswide: i32, // number of tiles in x direction, e.g. 8
    tileshigh: i32, // number of tiles in y direction, e.g. 8
    tilewidth: i32,  // width  of single tile in pixels, e.g. 32
    tileheight: i32, // height of single tile in pixels, e.g. 32
    layers: Vec<PyxelLayer>,
}

#[derive(Deserialize)]
struct PyxelLayer {
    // name: String,
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
}

fn pre_setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let pyxel_handle: Handle<PyxelFile> = asset_server.load("tiles.json");
    let tileset_handle: Handle<Image> = asset_server.load("tiles.png");

    commands.insert_resource(TileAssets {
        pyxel_file: pyxel_handle,
        tileset: tileset_handle,
    });
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
    pyxel_file_assets: Res<Assets<PyxelFile>>,
    tile_assets: Res<TileAssets>,
) {


    let pyxel_handle: Handle<PyxelFile> = asset_server.load("tiles.json");
    let pyxel_file = pyxel_file_assets.get(&pyxel_handle).unwrap();

    let layer = pyxel_file.layers.iter().find(|layer| layer.number == 0).unwrap();

    let min_tile = layer.tiles.iter()
        .filter(|tile| tile.tile != -1)
        .map(|tile| (tile.x, tile.y))
        .fold((i32::MAX, i32::MAX), |(min_x, min_y), (x, y)| (min(min_x, x), min(min_y, y)));
    let max_tile = layer.tiles.iter()
        .filter(|tile| tile.tile != -1)
        .map(|tile| (tile.x, tile.y))
        .fold((i32::MIN, i32::MIN), |(max_x, max_y), (x, y)| (max(max_x, x), max(max_y, y)));

    let mut tiles = MultiVec::new(-1, (max_tile.0 - min_tile.0 + 1) as usize, (max_tile.1 - min_tile.1 + 1) as usize);
    for tile in layer.tiles.iter() {
        if tile.tile != -1 {
            *(tiles.get_mut((tile.x - min_tile.0) as usize, (tile.y - min_tile.1) as usize).unwrap()) = tile.tile;
        }
    }

    // TODO: call david

    // checkerboard example
    let mut map_data: MultiVec<i32> = MultiVec::new(-1, 4, 4);
    for x in 0..4 {
        for y in 0..4 {
            if let Some(val) = map_data.get_mut(x, y) {
                *val = ((x + y) % 2) as i32;
            } else {
                panic!("out of bounds");
            }
        }
    }

    let texture_handle = asset_server.load("tiles.png");
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        Vec2::new(pyxel_file.tilewidth as f32, pyxel_file.tileheight as f32),
        pyxel_file.tileswide as usize,
        pyxel_file.tileshigh as usize, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    for y in 0..map_data.h {
        for x in 0..map_data.w {
            let tile = map_data.get(x, y).unwrap();
            if *tile != -1 {
                commands.spawn(SpriteSheetBundle {
                    texture_atlas: texture_atlas_handle.clone(),
                    sprite: TextureAtlasSprite::new(*tile as usize),
                    transform: Transform::from_translation(Vec3::new(x as f32, y as f32, 0.0)),
                    ..Default::default()
                });
            }
        }
    }

}
