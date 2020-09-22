use nalgebra_glm::TVec2;
use winit::event::TouchPhase;

#[derive(Debug, Clone)]
pub struct UITouch {
    pub id: u64,
    pub location: TVec2<u32>,
    pub phase: TouchPhase,
}
