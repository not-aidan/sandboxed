pub const WORLD_SIZE: u32 = 100;
pub const GRAVITY: f32 = 0.3;

pub struct Cell {
    element: CellElement,
}

#[derive(Copy, Clone, PartialEq)]
pub enum CellElement {
    Air,
    Sand(f32),
}

impl CellElement {
    fn push_color(&self, pixels: &mut Vec<u8>) {
        match self {
            Self::Air => {
                pixels.push(255);
                pixels.push(255);
                pixels.push(255);
                pixels.push(255);
            }
            Self::Sand(..) => {
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
    cells: [[CellElement; WORLD_SIZE as usize]; WORLD_SIZE as usize],
}

impl Default for World {
    fn default() -> Self {
        Self {
            cells: [[CellElement::Air; WORLD_SIZE as usize]; WORLD_SIZE as usize],
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
                self.update_cell(x, y, self.cells[y][x]);
            }
        }
    }

    fn update_cell(&mut self, x: usize, mut y: usize, cell: CellElement) {
        if let CellElement::Sand(mut velocity) = cell {
            velocity += GRAVITY;
            let mut distance = velocity.floor() as u32;
            while y > 0 && distance > 0 {
                if y == 0 {
                    return;
                }

                if self.get_cell(x, y - 1) == Some(CellElement::Air) {
                    self.set_cell(x, y, CellElement::Air);
                    self.set_cell(x, y - 1, CellElement::Sand(velocity));
                } else if self.get_cell(x + 1, y - 1) == Some(CellElement::Air) {
                    self.set_cell(x, y, CellElement::Air);
                    self.set_cell(x + 1, y - 1, CellElement::Sand(velocity));
                } else if x > 0 && self.get_cell(x - 1, y - 1) == Some(CellElement::Air) {
                    self.set_cell(x, y, CellElement::Air);
                    self.set_cell(x - 1, y - 1, CellElement::Sand(velocity));
                } else {
                    velocity = 0.0;
                }

                y -= 1;
                distance -= 1;
            }
        }
    }

    pub fn get_cell(&self, x: usize, y: usize) -> Option<CellElement> {
        if let Some(row) = self.cells.get(y) {
            return row.get(x).copied();
        }
        None
    }

    pub fn set_cell(&mut self, x: usize, y: usize, cell: CellElement) {
        self.cells[y][x] = cell;
    }
}
