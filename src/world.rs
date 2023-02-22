use crate::renderer::Vertex;

pub const WORLD_SIZE: u32 = 18;

#[allow(dead_code)]
pub struct World {
    cells: [[bool; WORLD_SIZE as usize]; WORLD_SIZE as usize],
}

impl Default for World {
    fn default() -> Self {
        Self {
            cells: [[true; WORLD_SIZE as usize]; WORLD_SIZE as usize],
        }
    }
}

impl World {
    pub fn verticies(&self) -> [Vertex; 4] {
        [
            Vertex {
                position: [0.5, 0.5],
            },
            Vertex {
                position: [0.5, -0.5],
            },
            Vertex {
                position: [-0.5, -0.5],
            },
            Vertex {
                position: [-0.5, 0.5],
            },
        ]
    }

    /// Returns pixels in sRGB
    pub fn pixels(&self) -> Vec<u8> {
        let mut pixels = Vec::<u8>::new();

        for row in self.cells.iter() {
            for cell in row.iter() {
                if *cell {
                    pixels.push(255);
                    pixels.push(255);
                    pixels.push(255);
                    pixels.push(255);
                } else {
                    pixels.push(0);
                    pixels.push(0);
                    pixels.push(0);
                    pixels.push(0);
                }
            }
        }

        pixels
    }
}
