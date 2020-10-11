use gfx_hal::pso::{Rect, Viewport};

pub fn convert_up_side_down(viewport: &Viewport) -> Viewport {
    Viewport {
        rect: Rect {
            x: viewport.rect.x,
            y: viewport.rect.y,
            w: viewport.rect.w,
            h: viewport.rect.h,
        },
        depth: viewport.depth.clone(),
    }
}
