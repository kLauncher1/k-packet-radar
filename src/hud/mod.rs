use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectCategory {
    Worker,
    CombatUnit,
    AirUnit,
    Building,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct TacticalObject {
    pub object_id: u16,
    pub player_id: u8,
    pub category: ObjectCategory,
    pub raw_x: u16,
    pub raw_y: u16,
    pub grid_x: u8, // normalized 0..255
    pub grid_y: u8, // normalized 0..255
    pub last_updated_frame: u32,
}

#[derive(Debug)]
pub struct RadarGrid {
    map_width: u16,
    map_height: u16,
    objects: HashMap<u16, TacticalObject>,
}

impl RadarGrid {
    pub fn new(map_width: u16, map_height: u16) -> Self {
        Self {
            map_width: map_width.max(1),
            map_height: map_height.max(1),
            objects: HashMap::new(),
        }
    }

    pub fn update_object(
        &mut self,
        object_id: u16,
        player_id: u8,
        category: ObjectCategory,
        raw_x: u16,
        raw_y: u16,
        frame: u32,
    ) {
        let grid_x = ((raw_x as u32 * 255) / self.map_width as u32).min(255) as u8;
        let grid_y = ((raw_y as u32 * 255) / self.map_height as u32).min(255) as u8;

        self.objects.insert(
            object_id,
            TacticalObject {
                object_id,
                player_id,
                category,
                raw_x,
                raw_y,
                grid_x,
                grid_y,
                last_updated_frame: frame,
            },
        );
    }

    pub fn get_objects_for_player(&self, player_id: u8) -> Vec<&TacticalObject> {
        self.objects
            .values()
            .filter(|o| o.player_id == player_id)
            .collect()
    }

    pub fn total_objects(&self) -> usize {
        self.objects.len()
    }

    pub fn render_ascii_minimap(&self, width: usize, height: usize) -> String {
        let mut grid = vec![vec!['.'; width]; height];
        for obj in self.objects.values() {
            let x = (obj.grid_x as usize * width) / 256;
            let y = (obj.grid_y as usize * height) / 256;
            if x < width && y < height {
                let ch = match obj.category {
                    ObjectCategory::Worker => 'w',
                    ObjectCategory::CombatUnit => 'u',
                    ObjectCategory::AirUnit => 'a',
                    ObjectCategory::Building => 'B',
                    ObjectCategory::Unknown => '?',
                };
                grid[y][x] = ch;
            }
        }

        grid.into_iter()
            .map(|row| row.into_iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
