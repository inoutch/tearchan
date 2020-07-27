use crate::core::screen::ScreenMode::Windowed;
use gfx_hal::pso::{Rect, Viewport};
use gfx_hal::window::Extent2D;
use nalgebra_glm::{vec2, Vec2};

#[derive(Clone, Debug)]
pub enum ScreenMode {
    FullScreenWindow,
    Windowed { resolutions: Vec<Extent2D> },
}

pub fn create_single_windowed(width: u32, height: u32) -> ScreenMode {
    Windowed {
        resolutions: vec![Extent2D { width, height }],
    }
}

#[derive(Clone, Debug)]
pub enum ScreenResolutionMode {
    Auto,
    FixWidth { width: f32 },
    FixHeight { height: f32 },
    Border { width: f32, height: f32 },
    Expand { width: f32, height: f32 },
}

impl ScreenResolutionMode {
    // calc logical size and viewport
    pub fn calc(&self, display_size: &DisplaySize) -> (Vec2, Viewport) {
        let aspect = display_size.physical.x / display_size.physical.y;
        match self {
            ScreenResolutionMode::Auto => (
                display_size.logical.clone_owned(),
                display_size.viewport.clone(),
            ),
            ScreenResolutionMode::FixWidth { width } => {
                (vec2(*width, width / aspect), display_size.viewport.clone())
            }
            ScreenResolutionMode::FixHeight { height } => (
                vec2(height * aspect, *height),
                display_size.viewport.clone(),
            ),
            ScreenResolutionMode::Border { width, height } => {
                let border_aspect = width / height;
                if border_aspect < aspect {
                    let dx = display_size.viewport.rect.w
                        - (display_size.viewport.rect.h as f32 * border_aspect) as i16;
                    // right-left border
                    (
                        vec2(height * aspect, *height),
                        Viewport {
                            rect: Rect {
                                x: dx / 2i16,
                                y: 0i16,
                                w: display_size.viewport.rect.w - dx,
                                h: display_size.viewport.rect.h,
                            },
                            depth: display_size.viewport.depth.clone(),
                        },
                    )
                } else {
                    let dy = display_size.viewport.rect.h
                        - (display_size.viewport.rect.w as f32 / border_aspect) as i16;
                    // up-down border
                    (
                        vec2(*width, width / aspect),
                        Viewport {
                            rect: Rect {
                                x: 0i16,
                                y: dy / 2i16,
                                w: display_size.viewport.rect.w,
                                h: display_size.viewport.rect.h - dy,
                            },
                            depth: display_size.viewport.depth.clone(),
                        },
                    )
                }
            }
            ScreenResolutionMode::Expand { width, height } => {
                (vec2(*width, *height), display_size.viewport.clone())
            }
        }
    }
}

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

#[cfg(test)]
mod test {
    use crate::core::screen::{DisplaySize, ScreenResolutionMode};
    use gfx_hal::pso::{Rect, Viewport};
    use nalgebra_glm::vec2;

    #[test]
    fn test_resolution_mode() {
        let display_size = DisplaySize {
            logical: vec2(900.0f32, 1200.0f32),
            physical: vec2(1800.0f32, 2400.0f32),
            viewport: Viewport {
                rect: Rect {
                    x: 0,
                    y: 0,
                    w: 900,
                    h: 1200,
                },
                depth: 0.0f32..1.0f32,
            },
        };

        assert_eq!(
            ScreenResolutionMode::Auto.calc(&display_size),
            (
                vec2(900.0f32, 1200.0f32),
                Viewport {
                    rect: Rect {
                        x: 0,
                        y: 0,
                        w: 900,
                        h: 1200,
                    },
                    depth: 0.0f32..1.0f32,
                }
            )
        );

        assert_eq!(
            ScreenResolutionMode::FixWidth { width: 300.0f32 }.calc(&display_size),
            (
                vec2(300.0f32, 400.0f32),
                Viewport {
                    rect: Rect {
                        x: 0,
                        y: 0,
                        w: 900,
                        h: 1200,
                    },
                    depth: 0.0f32..1.0f32,
                }
            )
        );

        assert_eq!(
            ScreenResolutionMode::FixHeight { height: 300.0f32 }.calc(&display_size),
            (
                vec2(225.0f32, 300.0f32),
                Viewport {
                    rect: Rect {
                        x: 0,
                        y: 0,
                        w: 900,
                        h: 1200,
                    },
                    depth: 0.0f32..1.0f32,
                }
            )
        );

        assert_eq!(
            ScreenResolutionMode::Border {
                width: 300.0f32,
                height: 300.0f32
            }
            .calc(&display_size),
            (
                vec2(300.0f32, 300.0f32),
                Viewport {
                    rect: Rect {
                        x: 0,
                        y: 150,
                        w: 900,
                        h: 1200 - 300,
                    },
                    depth: 0.0f32..1.0f32,
                }
            )
        );

        assert_eq!(
            ScreenResolutionMode::Border {
                width: 300.0f32,
                height: 600.0f32
            }
            .calc(&display_size),
            (
                vec2(300.0f32, 600.0f32),
                Viewport {
                    rect: Rect {
                        x: 150,
                        y: 0,
                        w: 900 - 300,
                        h: 1200,
                    },
                    depth: 0.0f32..1.0f32,
                }
            )
        );
    }
}
