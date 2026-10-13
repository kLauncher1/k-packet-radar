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

    pub fn export_svg(&self, width: u32, height: u32) -> String {
        let mut out = String::with_capacity(4096);
        out.push_str(&format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\" width=\"100%\" height=\"{height}\" style=\"background:#0d1117;border-radius:6px;font-family:monospace\">\n"
        ));

        // Background tactical grid
        out.push_str("  <g opacity=\"0.2\" stroke=\"#00E5FF\" stroke-width=\"1\">\n");
        let step = (width / 8).max(1);
        for x in (0..width).step_by(step as usize) {
            out.push_str(&format!(
                "    <line x1=\"{x}\" y1=\"0\" x2=\"{x}\" y2=\"{height}\"/>\n"
            ));
        }
        let step_y = (height / 8).max(1);
        for y in (0..height).step_by(step_y as usize) {
            out.push_str(&format!(
                "    <line x1=\"0\" y1=\"{y}\" x2=\"{width}\" y2=\"{y}\"/>\n"
            ));
        }
        out.push_str("  </g>\n");

        // Radar center crosshair & concentric circles
        let cx = width / 2;
        let cy = height / 2;
        let r_max = (cx.min(cy) as f32 * 0.9) as u32;
        out.push_str(&format!(
            "  <circle cx=\"{cx}\" cy=\"{cy}\" r=\"{r_max}\" fill=\"none\" stroke=\"#00E5FF\" stroke-width=\"1\" opacity=\"0.4\"/>\n"
        ));
        let r_mid = r_max / 2;
        out.push_str(&format!(
            "  <circle cx=\"{cx}\" cy=\"{cy}\" r=\"{r_mid}\" fill=\"none\" stroke=\"#00FF9D\" stroke-width=\"1\" opacity=\"0.3\" stroke-dasharray=\"4,4\"/>\n"
        ));

        // Render tactical objects
        for obj in self.objects.values() {
            let px = (obj.grid_x as u32 * width) / 256;
            let py = (obj.grid_y as u32 * height) / 256;
            let (color, radius) = match obj.category {
                ObjectCategory::Worker => ("#00FF9D", 3),
                ObjectCategory::CombatUnit => ("#00E5FF", 4),
                ObjectCategory::AirUnit => ("#7C4DFF", 5),
                ObjectCategory::Building => ("#FFB300", 6),
                ObjectCategory::Unknown => ("#8b949e", 3),
            };
            out.push_str(&format!(
                "  <circle cx=\"{px}\" cy=\"{py}\" r=\"{radius}\" fill=\"{color}\" opacity=\"0.9\"/>\n"
            ));
        }

        // Overlay status header
        out.push_str(&format!(
            "  <text x=\"16\" y=\"24\" fill=\"#00E5FF\" font-size=\"14\" font-weight=\"bold\">TACTICAL RADAR [{obj_count} UNITS DETECTED]</text>\n",
            obj_count = self.objects.len()
        ));
        out.push_str("</svg>\n");
        out
    }
}
