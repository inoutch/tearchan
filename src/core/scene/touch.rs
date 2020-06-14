use nalgebra_glm::TVec2;
use winit::event::TouchPhase;

#[derive(Debug)]
pub struct Touch {
    pub id: u64,
    pub location: TVec2<u32>,
    pub phase: TouchPhase,
}
