use crate::core::graphic::image::Image;
use gfx_hal::adapter::MemoryType;
use gfx_hal::command::CommandBuffer;
use gfx_hal::device::Device;
use gfx_hal::format::Swizzle;
use gfx_hal::memory::Segment;
use gfx_hal::pool::CommandPool;
use gfx_hal::queue::{CommandQueue, QueueGroup};
use gfx_hal::{Backend, Limits};
use std::mem::ManuallyDrop;
use std::rc::{Rc, Weak};

const COLOR_RANGE: gfx_hal::image::SubresourceRange = gfx_hal::image::SubresourceRange {
    aspects: gfx_hal::format::Aspects::COLOR,
    levels: 0..1,
    layers: 0..1,
};

pub struct TextureCommon<B: Backend> {
    device: Weak<B::Device>,
    image: ManuallyDrop<B::Image>,
    image_memory: ManuallyDrop<B::Memory>,
    image_view: ManuallyDrop<B::ImageView>,
    sampler: ManuallyDrop<B::Sampler>,
}

impl<B: Backend> TextureCommon<B> {
    pub fn new(
        device: &Rc<B::Device>,
        command_pool: &mut B::CommandPool,
        queue_group: &mut QueueGroup<B>,
        memory_types: &[MemoryType],
        limits: &Limits,
        image_raw: &Image,
    ) -> TextureCommon<B> {
        let image_stride = image_raw.stride;
        let width = image_raw.size().x;
        let height = image_raw.size().y;
        let kind = gfx_hal::image::Kind::D2(
            width as gfx_hal::image::Size,
            height as gfx_hal::image::Size,
            1,
            1,
        );

        let non_coherent_alignment = limits.non_coherent_atom_size as u64;
        let row_alignment_mask = limits.optimal_buffer_copy_pitch_alignment as u32 - 1;
        let row_pitch = (width * image_stride as u32 + row_alignment_mask) & !row_alignment_mask;
        let upload_size = (height * row_pitch) as u64;
        let padded_upload_size = ((upload_size + non_coherent_alignment - 1)
            / non_coherent_alignment)
            * non_coherent_alignment;

        let mut image_upload_buffer = unsafe {
            device.create_buffer(padded_upload_size, gfx_hal::buffer::Usage::TRANSFER_SRC)
        }
        .unwrap();
        let buffer_req = unsafe { device.get_buffer_requirements(&image_upload_buffer) };
        let upload_type = memory_types
            .iter()
            .enumerate()
            .position(|(id, mem_type)| {
                buffer_req.type_mask & (1 << id) as u64 != 0
                    && mem_type
                        .properties
                        .contains(gfx_hal::memory::Properties::CPU_VISIBLE)
            })
            .unwrap()
            .into();

        let image_mem_requirements =
            unsafe { device.get_buffer_requirements(&image_upload_buffer) };

        // copy image data into staging buffer
        let image_upload_memory = unsafe {
            let memory = device
                .allocate_memory(upload_type, image_mem_requirements.size)
                .unwrap();
            device
                .bind_buffer_memory(&memory, 0, &mut image_upload_buffer)
                .unwrap();
            let mapping = device.map_memory(&memory, Segment::ALL).unwrap();
            for y in 0..height as usize {
                let row = image_raw.row(y);
                std::ptr::copy_nonoverlapping(
                    row.as_ptr(),
                    mapping.offset(y as isize * row_pitch as isize),
                    width as usize * image_stride,
                );
            }
            device
                .flush_mapped_memory_ranges(std::iter::once((&memory, Segment::ALL)))
                .unwrap();
            device.unmap_memory(&memory);
            memory
        };

        let mut image = ManuallyDrop::new(
            unsafe {
                device.create_image(
                    kind,
                    1,
                    gfx_hal::format::Format::Rgba8Srgb,
                    gfx_hal::image::Tiling::Optimal,
                    gfx_hal::image::Usage::TRANSFER_DST | gfx_hal::image::Usage::SAMPLED,
                    gfx_hal::image::ViewCapabilities::empty(),
                )
            }
            .unwrap(),
        );
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
        let image_memory = ManuallyDrop::new(
            unsafe { device.allocate_memory(device_type, image_req.size) }.unwrap(),
        );
        unsafe { device.bind_image_memory(&image_memory, 0, &mut image) }.unwrap();
        let image_view = ManuallyDrop::new(
            unsafe {
                device.create_image_view(
                    &image,
                    gfx_hal::image::ViewKind::D2,
                    gfx_hal::format::Format::Rgba8Srgb,
                    Swizzle::NO,
                    COLOR_RANGE.clone(),
                )
            }
            .unwrap(),
        );

        let sampler = ManuallyDrop::new(
            unsafe {
                device.create_sampler(&gfx_hal::image::SamplerDesc::new(
                    gfx_hal::image::Filter::Linear,
                    gfx_hal::image::WrapMode::Clamp,
                ))
            }
            .expect("Can't create sampler"),
        );

        // copy buffer to texture
        let copy_fence = device.create_fence(false).expect("Could not create fence");
        unsafe {
            let mut cmd_buffer = command_pool.allocate_one(gfx_hal::command::Level::Primary);
            cmd_buffer.begin_primary(gfx_hal::command::CommandBufferFlags::ONE_TIME_SUBMIT);

            let image_barrier = gfx_hal::memory::Barrier::Image {
                states: (
                    gfx_hal::image::Access::empty(),
                    gfx_hal::image::Layout::Undefined,
                )
                    ..(
                        gfx_hal::image::Access::TRANSFER_WRITE,
                        gfx_hal::image::Layout::TransferDstOptimal,
                    ),
                target: &*image,
                families: None,
                range: COLOR_RANGE.clone(),
            };

            cmd_buffer.pipeline_barrier(
                gfx_hal::pso::PipelineStage::TOP_OF_PIPE..gfx_hal::pso::PipelineStage::TRANSFER,
                gfx_hal::memory::Dependencies::empty(),
                &[image_barrier],
            );

            cmd_buffer.copy_buffer_to_image(
                &image_upload_buffer,
                &image,
                gfx_hal::image::Layout::TransferDstOptimal,
                &[gfx_hal::command::BufferImageCopy {
                    buffer_offset: 0,
                    buffer_width: row_pitch / (image_stride as u32),
                    buffer_height: height as u32,
                    image_layers: gfx_hal::image::SubresourceLayers {
                        aspects: gfx_hal::format::Aspects::COLOR,
                        level: 0,
                        layers: 0..1,
                    },
                    image_offset: gfx_hal::image::Offset { x: 0, y: 0, z: 0 },
                    image_extent: gfx_hal::image::Extent {
                        width,
                        height,
                        depth: 1,
                    },
                }],
            );

            let image_barrier = gfx_hal::memory::Barrier::Image {
                states: (
                    gfx_hal::image::Access::TRANSFER_WRITE,
                    gfx_hal::image::Layout::TransferDstOptimal,
                )
                    ..(
                        gfx_hal::image::Access::SHADER_READ,
                        gfx_hal::image::Layout::ShaderReadOnlyOptimal,
                    ),
                target: &*image,
                families: None,
                range: COLOR_RANGE.clone(),
            };
            cmd_buffer.pipeline_barrier(
                gfx_hal::pso::PipelineStage::TRANSFER..gfx_hal::pso::PipelineStage::FRAGMENT_SHADER,
                gfx_hal::memory::Dependencies::empty(),
                &[image_barrier],
            );

            cmd_buffer.finish();

            queue_group.queues[0].submit_without_semaphores(Some(&cmd_buffer), Some(&copy_fence));

            device
                .wait_for_fence(&copy_fence, !0)
                .expect("Can't wait for fence");

            device.destroy_fence(copy_fence);
            device.free_memory(image_upload_memory);
            device.destroy_buffer(image_upload_buffer);
        }

        TextureCommon {
            device: Rc::downgrade(device),
            image,
            image_memory,
            image_view,
            sampler,
        }
    }

    pub fn sampler(&self) -> &B::Sampler {
        &self.sampler
    }

    pub fn image_view(&self) -> &B::ImageView {
        &self.image_view
    }
}

impl<B: Backend> Drop for TextureCommon<B> {
    fn drop(&mut self) {
        if let Some(device) = self.device.upgrade() {
            unsafe {
                device.destroy_sampler(ManuallyDrop::into_inner(std::ptr::read(&self.sampler)));
                device
                    .destroy_image_view(ManuallyDrop::into_inner(std::ptr::read(&self.image_view)));
                device.free_memory(ManuallyDrop::into_inner(std::ptr::read(&self.image_memory)));
                device.destroy_image(ManuallyDrop::into_inner(std::ptr::read(&self.image)));
            }
        }
    }
}
