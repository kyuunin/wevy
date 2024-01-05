use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct PyxelFile {
    tileswide: i32, // number of tiles in x direction, e.g. 8
    tileshigh: i32, // number of tiles in y direction, e.g. 8
    tileswidth: i32,  // width  of single tile in pixels, e.g. 32
    tilesheight: i32, // height of single tile in pixels, e.g. 32
    layers: Vec<PyxelLayer>,
}

#[derive(Serialize, Deserialize)]
struct PyxelLayer {
    name: String,
    number: i32,
    tiles: Vec<PyxelTile>,
}

#[derive(Serialize, Deserialize)]
struct PyxelTile {
    x: i32,
    y: i32,
    tile: i32, // index of tile in tileset, or -1 for empty / custom tile
    index: i32, // not sure, maybe y * width + x?
    flipX: bool,
    rot: i32, // 0, 1, 2, 3
}



fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let json_handle = asset_server.load("tiles.json");
    let json_bytes = asset_server.get_bytes(&json_handle).unwrap();
    let pyxel_file: PyxelFile = serde_json::from_slice(&json_bytes).unwrap();

    let layer = pyxel_file.layers.iter().find(|layer| layer.number == 0).unwrap();

    let min_tile = layer.iter()
        .filter(|tile| tile.tile != -1)
        .map(|tile| (tile.x, tile.y))
        .fold((i32::MAX, i32::MAX), |(min_x, min_y), (x, y)| (min(min_x, x), min(min_y, y)));
    let max_tile = layer.iter()
        .filter(|tile| tile.tile != -1)
        .map(|tile| (tile.x, tile.y))
        .fold((i32::MIN, i32::MIN), |(max_x, max_y), (x, y)| (max(max_x, x), max(max_y, y)));

    let mut tiles = MultiVec::new(-1, max_tile.0 - min_tile.0 + 1, max_tile.1 - min_tile.1 + 1);
    for tile in layer.iter() {
        if tile.tile != -1 {
            tiles.set(tile.x - min_tile.0, tile.y - min_tile.1, tile.tile);
        }
    }

    // TODO: call david

    // checkerboard example
    let mut map_data: MultiVec<i32> = MultiVec::new(-1, 4, 4);
    for x in 0..4 {
        for y in 0..4 {
            map_data.set(x, y, (x + y) % 2);
        }
    }

    let texture_handle = asset_server.load("tiles.png");



}
