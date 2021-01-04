use gfx_hal::command::{
    ClearColor, ClearDepthStencil, ClearValue, CommandBufferFlags, Level, SubpassContents,
};
use gfx_hal::image::{Extent, Layout};
use gfx_hal::pass::{Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp, SubpassDesc};
use gfx_hal::pso::{PipelineStage, Rect};
use gfx_hal::queue::Submission;
use std::iter;
use std::iter::Once;
use tearchan::engine::Engine;
use tearchan::engine_config::EngineStartupConfigBuilder;
use tearchan::scene::context::{SceneContext, SceneRenderContext};
use tearchan::scene::factory::SceneFactory;
use tearchan::scene::{Scene, SceneControlFlow};
use tearchan_gfx::{CommandBuffer, Semaphore};
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
        let gfx = context.gfx();
        let color_format = gfx.find_support_format();
        let depth_stencil_format = frame.depth_texture().format().clone();
        let extent = Extent {
            width: gfx.swapchain_desc().config.extent.width,
            height: gfx.swapchain_desc().config.extent.height,
            depth: 1,
        };
        let render_area = Rect {
            x: 0,
            y: 0,
            w: context.gfx().swapchain_desc().config.extent.width as _,
            h: context.gfx().swapchain_desc().config.extent.height as _,
        };

        frame
            .submission_complete_fence()
            .wait_for_fence(!0)
            .unwrap();
        frame.submission_complete_fence().reset_fence();

        let command_buffer = frame.command_pool().allocate_one(Level::Primary);
        command_buffer.begin_primary(CommandBufferFlags::ONE_TIME_SUBMIT);

        let render_pass = {
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
            gfx.device()
                .create_render_pass(&[attachment, depth_attachment], &[subpass], &[])
        };
        let framebuffer = context.gfx().device().create_framebuffer(
            &render_pass,
            vec![frame.depth_texture().image_view()],
            extent,
        );

        command_buffer.begin_render_pass(
            &render_pass,
            &framebuffer,
            render_area,
            &[
                ClearValue {
                    color: ClearColor {
                        float32: [0.3, 0.3, 0.3, 1.0],
                    },
                },
                ClearValue {
                    depth_stencil: ClearDepthStencil {
                        depth: 1.0f32,
                        stencil: 0,
                    },
                },
            ],
            SubpassContents::Inline,
        );
        command_buffer.end_render_pass();
        command_buffer.finish();
        let submission: Submission<
            Once<&CommandBuffer>,
            Vec<(&Semaphore, PipelineStage)>,
            Vec<&Semaphore>,
        > = Submission {
            command_buffers: iter::once(&command_buffer),
            wait_semaphores: vec![],
            signal_semaphores: vec![frame.submission_complete_semaphore()],
        };
        gfx.queue()
            .submit(submission, Some(frame.submission_complete_fence()));

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
