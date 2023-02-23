pub const WORLD_SIZE: u32 = 18;

#[derive(Copy, Clone)]
enum CellType {
    Air,
    Sand,
}

impl CellType {
    fn push_color(&self, pixels: &mut Vec<u8>) {
        match self {
            Self::Air => {
                pixels.push(255);
                pixels.push(255);
                pixels.push(255);
                pixels.push(255);
            }
            Self::Sand => {
                pixels.push(0);
                pixels.push(0);
                pixels.push(0);
                pixels.push(0);
            }
        }
    }
}

#[allow(dead_code)]
pub struct World {
    cells: [[CellType; WORLD_SIZE as usize]; WORLD_SIZE as usize],
}

impl Default for World {
    fn default() -> Self {
        let mut world = Self {
            cells: [[CellType::Air; WORLD_SIZE as usize]; WORLD_SIZE as usize],
        };
        world.cells[5][5] = CellType::Sand;
        world
    }
}

impl World {
    /// Returns pixels in sRGB
    pub fn pixels(&self) -> Vec<u8> {
        let mut pixels = Vec::<u8>::new();

        for row in self.cells.iter() {
            for cell in row.iter() {
                cell.push_color(&mut pixels);
            }
        }

        pixels
    }
}
