use gfx_hal::image::Layout;
use gfx_hal::pass::{Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp, SubpassDesc};
use tearchan::engine::Engine;
use tearchan::engine_config::EngineStartupConfigBuilder;
use tearchan::scene::context::{SceneContext, SceneRenderContext};
use tearchan::scene::factory::SceneFactory;
use tearchan::scene::{Scene, SceneControlFlow};
use winit::event::WindowEvent;
use winit::window::WindowBuilder;

struct QuadScene {}

impl QuadScene {
    fn factory() -> SceneFactory {
        |_context, _| Box::new(QuadScene {})
    }
}

impl Scene for QuadScene {
    fn update(&mut self, _context: &mut SceneContext, _event: WindowEvent) -> SceneControlFlow {
        SceneControlFlow::None
    }

    fn render(&mut self, context: &mut SceneRenderContext) -> SceneControlFlow {
        let frame = context.gfx_rendering().frame();
        let color_format = context.gfx().find_support_format();
        let depth_stencil_format = frame.depth_texture().format().clone();

        {
            let color_load_op = AttachmentLoadOp::Clear;
            let depth_load_op = AttachmentLoadOp::Clear;
            let attachment = Attachment {
                format: Some(color_format),
                samples: 1,
                ops: AttachmentOps::new(color_load_op, AttachmentStoreOp::Store),
                stencil_ops: AttachmentOps::DONT_CARE,
                layouts: Layout::Undefined..Layout::Present,
            };
            let depth_attachment = Attachment {
                format: Some(depth_stencil_format),
                samples: 1,
                ops: AttachmentOps::new(depth_load_op, AttachmentStoreOp::Store),
                stencil_ops: AttachmentOps::DONT_CARE,
                layouts: Layout::Undefined..Layout::DepthStencilAttachmentOptimal,
            };
            let subpass = SubpassDesc {
                colors: &[(0, Layout::ColorAttachmentOptimal)],
                depth_stencil: Some(&(1, Layout::DepthStencilAttachmentOptimal)),
                inputs: &[],
                resolves: &[],
                preserves: &[],
            };
            context.gfx().device().create_render_pass(
                &[attachment, depth_attachment],
                &[subpass],
                &[],
            );
        }

        SceneControlFlow::None
    }
}

pub fn main() {
    let window_builder = WindowBuilder::new().with_title("quad");
    let startup_config = EngineStartupConfigBuilder::default()
        .window_builder(window_builder)
        .scene_factory(QuadScene::factory())
        .build()
        .unwrap();
    let engine = Engine::new(startup_config);
    engine.run();
}
