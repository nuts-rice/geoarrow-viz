use dashmap::DashMap;
use crate::model::{Tile, GeoArrowResult}; 
pub struct TileInfo {
    pub id: u32,
    pub x: u32,
    pub y: u32,
    pub z: u8,
}

impl TileInfo {
    pub fn new(id: u32, x: u32, y: u32, z: u8) -> Self {
        Self {id, x, y, z}
    }

}

pub struct TileCache {
    tiles: DashMap<u32, Tile>,
    access_order: Vec<u32>,
    max_size: usize,
    current_size: usize,

}

impl TileCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            tiles: DashMap::new(),
            max_size,
            current_size: 0,
        }
    }
    pub fn get(&self, id: &u32) -> Option<Tile> {
        self.tiles.get(id).map(|entry| entry.value().clone())
    }

    pub fn insert(&mut self, id: u32, tile: Tile) {
        if self.current_size >= self.max_size {
            self.evict_oldest()
        }
        todo!()
    }

    async fn evict_oldest(&mut self) -> GeoArrowResult<()> {
        if let Some(oldest_id) = self.access_order.first() {
            self.tiles.remove(oldest_id);
            self.access_order.remove(0);
            self.current_size -= 1;


        }
        Ok(())
    }


        

    fn memory_usage(&self) -> usize {
        self.current_size
    }
}






#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_tile_cache() {
        let cache = TileCache::new(10);
        let tile_info1  = TileInfo::new(0, 0, 0, 1) ;
        let tile_info2 = TileInfo::new(1, 1, 1, 1);

    }
}
