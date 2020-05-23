use crate::core::screen::ScreenMode::Windowed;
use gfx_hal::window::Extent2D;

#[derive(Clone)]
pub enum ScreenMode {
    FullScreenWindow,
    Windowed { resolutions: Vec<Extent2D> },
}

pub fn create_single_windowed(width: u32, height: u32) -> ScreenMode {
    Windowed {
        resolutions: vec![Extent2D { width, height }],
    }
}
