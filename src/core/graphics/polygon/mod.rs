pub struct Polygon<T> {
    base: T,
}

impl<T> Polygon<T> {
    pub fn new(base: T) -> Polygon<T> {
        Polygon { base }
    }
}

pub struct BatchTest<T> {
    polygons: Vec<Polygon<T>>,
}

impl<T> BatchTest<T> {
    pub fn new() -> Self {
        BatchTest { polygons: vec![] }
    }

    pub fn add(&mut self, polygon: Polygon<T>) {
        self.polygons.push(polygon);
    }
}

#[cfg(test)]
mod tests {
    use crate::core::graphics::polygon::{Polygon, BatchTest};

    type PolygonI32 = Polygon<i32>;
    type PolygonF32 = Polygon<f32>;

    #[test]
    fn test() {
        let polygon_i32: PolygonI32 = Polygon::new(12);
        let polygon_f32: PolygonF32 = Polygon::new(0.12f32);

        let mut batch = BatchTest::new();
        batch.add(polygon_i32);
    }
}
