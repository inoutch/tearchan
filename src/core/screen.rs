use gfx_hal::window::Extent2D;

#[derive(Clone)]
pub enum ScreenMode {
    FullScreenWindow,
    Windowed { resolutions: Vec<Extent2D> },
}
