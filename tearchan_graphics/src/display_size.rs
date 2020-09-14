use crate::screen::ScreenResolutionMode;
use gfx_hal::pso::Viewport;
use nalgebra_glm::Vec2;

#[derive(Clone)]
pub struct DisplaySize {
    pub logical: Vec2,
    pub physical: Vec2,
    pub viewport: Viewport,
}

impl DisplaySize {
    pub fn update(&mut self, resolution_mode: &ScreenResolutionMode) {
        let (logical, viewport) = resolution_mode.calc(self);
        self.logical = logical;
        self.viewport = viewport;
    }
}
