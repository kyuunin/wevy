use bevy::prelude::*;
#[derive(Debug, Copy, Clone)]
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

#[derive(Component, Debug, Reflect, Copy, Clone)]
pub struct GameObject {
    pub tile_id: i32,
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

