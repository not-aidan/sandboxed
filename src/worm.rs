use nalgebra::Vector2;

use crate::world;

pub struct Worm {
    pub head: WormSegment,
    pub segments: Vec<WormSegment>,
    pub segment_length: f32,
    pub speed: f32,
}

impl Worm {
    /// creates a straight room facing direction; has to be normalized
    pub fn new(
        segment_count: u8,
        mut position: Vector2<f32>,
        direction: Vector2<f32>,
        segment_length: f32,
        speed: f32,
    ) -> Self {
        let head = WormSegment(position);
        let mut segments = Vec::<WormSegment>::new();

        for _i in 0..segment_count {
            let next_position = (position - direction) * segment_length;
            segments.push(WormSegment(next_position));
            position = next_position;
        }

        Self {
            head,
            segment_length,
            segments,
            speed,
        }
    }

    pub fn move_to(&mut self, position: Vector2<f32>) {
        self.head.0 = position;
        let mut head = self.head;
        for segment in self.segments.iter_mut() {
            let normal = (segment.0 - head.0).normalize();
            segment.0 = (normal * self.segment_length) + head.0;
            head = *segment;
        }
    }

    pub fn direction(&self) -> Option<Vector2<f32>> {
        if let Some(neck) = self.segments.get(0) {
            return Some((self.head.0 - neck.0).normalize());
        }
        None
    }

    pub fn step_ai(&mut self, delta: f32) {
        // move straight for now
        if let Some(direction) = self.direction() {
            self.move_to(self.head.0 + direction * self.speed * delta);
        }
    }
}

#[derive(Clone, Copy)]
pub struct WormSegment(pub Vector2<f32>);

impl WormSegment {
    pub fn force(&self) -> world::Force {
        world::Force {
            position: self.0,
            strength: 120.0,
            max_distance_squared: 900.0,
            min_distance_squared: 80.0,
        }
    }
}
