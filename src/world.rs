use nalgebra::Vector2;
use rand::Rng;

pub const WORLD_SIZE: u32 = 100;
pub const GRAVITY: Vector2<f32> = Vector2::new(0.0, -0.3);
const AIR_FRICTION: f32 = 0.25;

pub type Coordinate = Vector2<u32>;

trait Difference<T> {
    fn difference(&self, other: &Self) -> T;
    fn as_f32(&self) -> Vector2<f32>;
    fn in_bounds(&self) -> bool;
}

impl Difference<Vector2<i32>> for Coordinate {
    fn difference(&self, other: &Self) -> Vector2<i32> {
        Vector2::new(
            self.x as i32 - other.x as i32,
            self.y as i32 - other.y as i32,
        )
    }

    fn as_f32(&self) -> Vector2<f32> {
        Vector2::new(self.x as f32, self.y as f32)
    }

    fn in_bounds(&self) -> bool {
        self.x < WORLD_SIZE && self.y < WORLD_SIZE
    }
}

trait Unit {
    fn unit_neighbors(&self) -> Option<[Vector2<i32>; 2]>;
}

impl Unit for Vector2<i32> {
    /// returns unit neighbors for sand simulation if the coordinate is a unit itself
    fn unit_neighbors(&self) -> Option<[Self; 2]> {
        if Self::new(0, 1) == *self {
            Some([Self::new(1, 1), Self::new(-1, 1)])
        } else if Self::new(1, 0) == *self {
            Some([Self::new(1, 1), Self::new(1, -1)])
        } else if Self::new(1, 1) == *self {
            Some([Self::new(1, 0), Self::new(0, 1)])
        } else if Self::new(-1, 1) == *self {
            Some([Self::new(-1, 0), Self::new(0, 1)])
        } else if Self::new(0, -1) == *self {
            Some([Self::new(-1, -1), Self::new(1, -1)])
        } else if Self::new(-1, 0) == *self {
            Some([Self::new(-1, -1), Self::new(-1, -1)])
        } else if Self::new(-1, -1) == *self {
            Some([Self::new(-1, 0), Self::new(0, -1)])
        } else if Self::new(1, -1) == *self {
            Some([Self::new(1, 0), Self::new(0, -1)])
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum CellElement {
    Air,
    Sand(Vector2<f32>),
}

impl CellElement {
    fn push_color(&self, pixels: &mut Vec<u8>) {
        match self {
            Self::Air => {
                pixels.push(0);
                pixels.push(0);
                pixels.push(255);
                pixels.push(255);
            }
            Self::Sand(..) => {
                pixels.push(255);
                pixels.push(255);
                pixels.push(0);
                pixels.push(255);
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

    pub fn update(&mut self, forces: &Vec<Force>) {
        for y in 0..WORLD_SIZE {
            for x in 0..WORLD_SIZE {
                self.update_cell(
                    Coordinate::new(x, y),
                    self.cells[y as usize][x as usize],
                    forces,
                );
            }
        }
    }

    fn update_cell(&mut self, mut coordinate: Coordinate, cell: CellElement, forces: &Vec<Force>) {
        if let CellElement::Sand(mut velocity) = cell {
            if velocity.magnitude_squared() > 1000.0 {
                println!("WARN:coordinate{coordinate}velocity{velocity}");
            }
            // forces
            velocity += GRAVITY;

            {
                let position = coordinate.as_f32();
                for force in forces.iter() {
                    let difference = force.position - position;
                    let distance_squared = difference.magnitude_squared();
                    if distance_squared >= force.min_distance_squared
                        && distance_squared <= force.max_distance_squared
                    {
                        velocity += difference.normalize() * (force.strength / distance_squared);
                    }
                }
            }
            // friction
            {
                if velocity.magnitude_squared() > AIR_FRICTION * AIR_FRICTION {
                    velocity -= velocity.normalize() * AIR_FRICTION;
                }
            }

            let destination: Coordinate;
            {
                let mut x = (coordinate.x as f32 + velocity.x).floor();
                let mut y = (coordinate.y as f32 + velocity.y).floor();

                // prevent overflows
                if x < 0.0 {
                    x = 0.0;
                    velocity.x = 0.0;
                }

                if y < 0.0 {
                    y = 0.0;
                    velocity.y = 0.0;
                }

                destination = Coordinate::new(x as u32, y as u32);
            }

            self.set_cell(&coordinate, CellElement::Sand(velocity));

            if destination == coordinate {
                return;
            }

            for step_coordinate in path(&coordinate, &destination).drain(..) {
                // check if blocked
                if let Some(CellElement::Sand(..)) = self.get_cell(&step_coordinate) {
                    // change trajectory to a random empty neighbor
                    let unit = step_coordinate.difference(&coordinate);
                    if let Some(mut neighbors) = unit.unit_neighbors() {
                        if rand::thread_rng().gen_bool(0.5) {
                            neighbors.swap(0, 1);
                        }

                        for neighbor in neighbors.iter() {
                            let neighbor_coordinate = Coordinate::new(
                                (coordinate.x as i32 + neighbor.x) as u32,
                                (coordinate.y as i32 + neighbor.y) as u32,
                            );

                            if !neighbor_coordinate.in_bounds()
                                || self.get_cell(&neighbor_coordinate) != Some(CellElement::Air)
                            {
                                continue;
                            }

                            self.swap_cells(&coordinate, &neighbor_coordinate);
                            return;
                        }
                    }

                    self.set_cell(&coordinate, CellElement::Sand(Vector2::zeros()));
                    break;
                }

                self.swap_cells(&coordinate, &step_coordinate);
                coordinate = step_coordinate;
            }
        }
    }

    pub fn swap_cells(&mut self, a_coordinate: &Coordinate, b_coordinate: &Coordinate) {
        if let Some(a) = self.get_cell(a_coordinate) {
            if let Some(b) = self.get_cell(b_coordinate) {
                self.set_cell(a_coordinate, b);
                self.set_cell(b_coordinate, a);
            }
        }
    }

    pub fn get_cell(&self, coordinate: &Coordinate) -> Option<CellElement> {
        if let Some(row) = self.cells.get(coordinate.y as usize) {
            return row.get(coordinate.x as usize).cloned();
        }
        None
    }

    pub fn set_cell(&mut self, coordinate: &Coordinate, cell: CellElement) {
        self.cells[coordinate.y as usize][coordinate.x as usize] = cell;
    }
}

pub struct Force {
    pub position: Vector2<f32>,
    pub strength: f32,
    pub min_distance_squared: f32,
    pub max_distance_squared: f32,
}

/// translated from https://gist.github.com/DavidMcLaughlin208/60e69e698e3858617c322d80a8f174e2
/// TODO optimize
fn path(start: &Vector2<u32>, end: &Vector2<u32>) -> Vec<Vector2<u32>> {
    if start == end {
        return Vec::new();
    }

    let mut path = Vec::<Vector2<u32>>::new();

    let matrix_x1 = start.x as i32;
    let matrix_y1 = start.y as i32;
    let matrix_x2 = end.x as i32;
    let matrix_y2 = end.y as i32;

    let x_diff = matrix_x1 - matrix_x2;
    let y_diff = matrix_y1 - matrix_y2;

    let x_diff_is_larger = x_diff.abs() > y_diff.abs();

    let x_modifier: i32 = if x_diff < 0 { 1 } else { -1 };
    let y_modifier: i32 = if y_diff < 0 { 1 } else { -1 };

    let longer_side_length = x_diff.abs().max(y_diff.abs());
    let shorter_side_length = x_diff.abs().min(y_diff.abs());

    let slope = if shorter_side_length == 0 || longer_side_length == 0 {
        0.0
    } else {
        shorter_side_length as f32 / longer_side_length as f32
    };

    let mut shorter_side_increase: i32;

    for i in 1..=longer_side_length {
        shorter_side_increase = (slope * i as f32).round() as i32;
        let x_increase: i32;
        let y_increase: i32;

        if x_diff_is_larger {
            x_increase = i;
            y_increase = shorter_side_increase;
        } else {
            y_increase = i;
            x_increase = shorter_side_increase;
        }

        let current_y = matrix_y1 + (y_increase * y_modifier);
        let current_x = matrix_x1 + (x_increase * x_modifier);

        //println!("x: {current_x}; y: {current_y}");
        path.push(Vector2::new(current_x as u32, current_y as u32));
    }

    path
}

#[cfg(test)]
mod tests {
    use nalgebra::Vector2;

    use super::path;

    fn test_path(from: Vector2<u32>, to: Vector2<u32>, between: Vec<Vector2<u32>>) {
        let path = path(&from, &to);
        if from == to {
            assert_eq!(path.len(), 0);
            return;
        }
        assert_eq!(path.len(), between.len() + 1);
        for i in 0..between.len() {
            assert_eq!(path[i], between[i]);
        }
        assert_eq!(path[between.len()], to);
    }

    #[test]
    fn path_works() {
        test_path(Vector2::new(0, 0), Vector2::new(0, 0), vec![]);
        test_path(Vector2::new(0, 0), Vector2::new(1, 0), vec![]);
        test_path(Vector2::new(0, 0), Vector2::new(0, 1), vec![]);
        test_path(Vector2::new(0, 0), Vector2::new(1, 1), vec![]);
        test_path(Vector2::new(1, 0), Vector2::new(0, 0), vec![]);
        test_path(Vector2::new(1, 1), Vector2::new(0, 0), vec![]);
        test_path(Vector2::new(0, 1), Vector2::new(0, 0), vec![]);

        test_path(
            Vector2::new(0, 0),
            Vector2::new(0, 3),
            vec![Vector2::new(0, 1), Vector2::new(0, 2)],
        );

        test_path(
            Vector2::new(0, 0),
            Vector2::new(3, 3),
            vec![Vector2::new(1, 1), Vector2::new(2, 2)],
        );
    }
}
