use crate::display_size::DisplaySize;
use crate::screen::ScreenMode::Windowed;
use gfx_hal::pso::{Rect, Viewport};
use nalgebra_glm::{vec2, TVec2, Vec2};

#[derive(Clone, Debug)]
pub enum ScreenMode {
    FullScreenWindow,
    Windowed { resolutions: Vec<TVec2<u32>> },
}

pub fn create_single_windowed(width: u32, height: u32) -> ScreenMode {
    Windowed {
        resolutions: vec![vec2(width, height)],
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
