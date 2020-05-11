use nalgebra_glm::vec3;

trait A {
    fn a(&self) -> u32;
}

struct AA {}

impl A for AA {
    fn a(&self) -> u32 {
        0
    }
}

struct Parent {
    parent: Box<dyn A>,
}

impl Parent {
    fn new() -> Parent {
        Parent {
            parent: Box::new(AA {}),
        }
    }
}

#[test]
fn sandbox() {
    assert!(vec3(2.0f32, 1.0f32, 1.0f32) == vec3(1.0f32, 1.0f32, 1.0f32));
}
