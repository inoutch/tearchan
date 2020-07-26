use gfx_hal::adapter::{Adapter, MemoryType};
use gfx_hal::device::Device;
use gfx_hal::format::{Format, Swizzle};
use gfx_hal::image::{Layout, ViewKind};
use gfx_hal::pass::{Attachment, AttachmentLoadOp, AttachmentOps, AttachmentStoreOp};
use gfx_hal::queue::QueueFamily;
use gfx_hal::window::Surface;
use gfx_hal::Backend;

pub fn create_render_pass<B: Backend>(
    device: &B::Device,
    surface_format: &Format,
    depth_stencil_format: &Format,
    first: bool,
) -> B::RenderPass {
    let load_op = if first {
        AttachmentLoadOp::Clear
    } else {
        AttachmentLoadOp::Load
    };

    let attachment = Attachment {
        format: Some(*surface_format),
        samples: 1,
        ops: AttachmentOps::new(load_op, AttachmentStoreOp::Store),
        stencil_ops: AttachmentOps::DONT_CARE,
        layouts: Layout::Undefined..Layout::Present,
    };
    let depth_attachment = Attachment {
        format: Some(*depth_stencil_format),
        samples: 1,
        ops: AttachmentOps::new(load_op, AttachmentStoreOp::Store),
        stencil_ops: AttachmentOps::DONT_CARE,
        layouts: Layout::Undefined..Layout::DepthStencilAttachmentOptimal,
    };
    let subpass = gfx_hal::pass::SubpassDesc {
        colors: &[(0, gfx_hal::image::Layout::ColorAttachmentOptimal)],
        depth_stencil: Some(&(1, gfx_hal::image::Layout::DepthStencilAttachmentOptimal)),
        inputs: &[],
        resolves: &[],
        preserves: &[],
    };

    unsafe { device.create_render_pass(&[attachment, depth_attachment], &[subpass], &[]) }.unwrap()
}

pub fn find_queue_family<'a, B: Backend>(
    adapter: &'a Adapter<B>,
    surface: &B::Surface,
) -> &'a B::QueueFamily {
    adapter
        .queue_families
        .iter()
        .find(|family| {
            surface.supports_queue_family(family) && family.queue_type().supports_graphics()
        })
        .unwrap()
}

pub fn create_depth_resources<B: Backend>(
    device: &B::Device,
    extent: gfx_hal::window::Extent2D,
    memory_types: &[MemoryType],
) -> (B::Image, B::Memory, B::ImageView, gfx_hal::format::Format) {
    let width = extent.width;
    let height = extent.height;
    let depth_stencil_format = gfx_hal::format::Format::D32SfloatS8Uint;
    let color_range = gfx_hal::image::SubresourceRange {
        aspects: gfx_hal::format::Aspects::COLOR,
        levels: 0..1,
        layers: 0..1,
    };
    let kind = gfx_hal::image::Kind::D2(
        width as gfx_hal::image::Size,
        height as gfx_hal::image::Size,
        1,
        1,
    );

    let mut image = unsafe {
        device.create_image(
            kind,
            1,
            depth_stencil_format,
            gfx_hal::image::Tiling::Optimal,
            gfx_hal::image::Usage::DEPTH_STENCIL_ATTACHMENT,
            gfx_hal::image::ViewCapabilities::empty(),
        )
    }
    .unwrap();
    let image_req = unsafe { device.get_image_requirements(&image) };

    let device_type = memory_types
        .iter()
        .enumerate()
        .position(|(id, memory_type)| {
            image_req.type_mask & (1 << id) as u64 != 0
                && memory_type
                    .properties
                    .contains(gfx_hal::memory::Properties::DEVICE_LOCAL)
        })
        .unwrap()
        .into();
    let image_memory = unsafe { device.allocate_memory(device_type, image_req.size) }.unwrap();
    unsafe { device.bind_image_memory(&image_memory, 0, &mut image) }.unwrap();
    let image_view = unsafe {
        device.create_image_view(
            &image,
            ViewKind::D2,
            depth_stencil_format,
            Swizzle::NO,
            color_range,
        )
    }
    .unwrap();

    (image, image_memory, image_view, depth_stencil_format)
}
