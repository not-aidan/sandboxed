pub const WORLD_SIZE: u32 = 18;

#[derive(Copy, Clone, PartialEq)]
pub enum CellType {
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
        Self {
            cells: [[CellType::Air; WORLD_SIZE as usize]; WORLD_SIZE as usize],
        }
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

    pub fn update(&mut self) {
        for y in 0usize..WORLD_SIZE as usize {
            for x in 0usize..WORLD_SIZE as usize { 
                let cell = self.cells[y][x];
                if y > 0 && cell == CellType::Sand && self.get_cell(x, y - 1) == Some(CellType::Air) {
                    self.set_cell(x, y, CellType::Air);
                    self.set_cell(x, y - 1, CellType::Sand);
                }
            }
        }
    }

    pub fn get_cell(&self, x: usize, y: usize) -> Option<CellType> {
        if let Some(row) = self.cells.get(y) {
            return row.get(x).copied();
        }
        None
    }

    pub fn set_cell(&mut self, x: usize, y: usize, cell: CellType) {
        self.cells[y][x] = cell;
    }
}
