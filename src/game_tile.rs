use bevy::prelude::*;
use crate::multi_vec::MultiVec;
#[derive(Component, Debug, Reflect, Clone, Copy)]

pub struct GameTile {
    pub tile_id: i32,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TileType {
    Water, Field, Mountain, Desert,
}

#[derive(Default, Resource)]
pub struct MapData(pub MultiVec<Option<Entity>>);

impl MapData {    
    pub fn get_tile_at_pos(&self, pos: Vec2, tiles: &Query<&GameTile>) -> Option<(usize, usize, GameTile)> {
        let pos = pos.round();
        if pos.x < -0.5 {return None}
        let x = pos.x as usize;
        if pos.y < -0.5 {return None}
        let y = pos.y as usize;
        let entity = self.0.get(x, y)?.as_ref()?;
        Some((x, y, *tiles.get_component::<GameTile>(*entity).ok()?))
    }
}

impl TileType {
    pub fn can_enter(self) -> bool{
        match self {
            TileType::Field    => true,
            TileType::Desert   => true,
            TileType::Water    => false,
            TileType::Mountain => false,
        }
    }
}

impl GameTile {
    pub fn top_left_type(&self) -> Option<TileType> {
        use TileType::*;
        match self.tile_id {
             0| 1| 2| 3| 8|11|15                                        => Some(Water),
             4| 5| 6| 7| 9|10|14|16|17|18|19|24|31|32|33|34|35|40|47|57 => Some(Field),
            20|21|22|23|25|26|30|43|44|45|48|51|56|59|63                => Some(Mountain),
            36|37|38|39|41|42|46|49|53|54|55|60|61|62                   => Some(Desert),
            _                                                           => None,
        }
    }
    
    pub fn top_right_type(&self) -> Option<TileType> {
        use TileType::*;
        match self.tile_id {
             0| 1| 2| 5|10|11|14                                        => Some(Water),
             3| 4| 6| 7| 8| 9|15|16|17|18|21|26|30|32|33|34|37|42|46|56 => Some(Field),
            19|20|22|23|24|25|31|43|44|45|49|53|57|61|62                => Some(Mountain),
            35|36|38|39|40|41|47|48|51|54|55|59|60|63                   => Some(Desert),
            _                                                           => None,
        }
    }
    pub fn bottom_left_type(&self) -> Option<TileType> {
        use TileType::*;
        match self.tile_id {
             0| 3| 4| 5| 7| 8|11                                        => Some(Water),
             1| 2| 6| 9|10|14|15|16|19|20|21|23|24|32|35|36|37|39|40|49 => Some(Field),
            17|18|22|25|26|30|31|43|48|51|55|56|59|60|61                => Some(Mountain),
            33|34|38|41|42|44|45|46|47|53|54|57|52|63                   => Some(Desert),
            _                                                           => None,
        }
    }
    
    pub fn bottom_right_type(&self) -> Option<TileType> {
        use TileType::*;
        match self.tile_id {
             2| 3| 4| 5| 6|10|11                                        => Some(Water),
             0| 1| 7| 8| 9|14|15|18|19|20|21|22|26|34|35|36|37|38|42|48 => Some(Field),
            16|17|23|24|25|30|31|45|49|53|54|57|59|60|61                => Some(Mountain),
            32|33|39|40|41|43|44|46|47|51|55|56|62|63                   => Some(Desert),
            _                                                           => None,
        }
    }
}


