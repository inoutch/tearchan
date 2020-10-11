use crate::hal::helper::find_memory_type;
use crate::hal::render_bundle::RenderBundleCommon;
use crate::image::Image;
use gfx_hal::command::{CommandBuffer, CommandBufferFlags, Level};
use gfx_hal::device::Device;
use gfx_hal::format::{Aspects, Format, Swizzle};
use gfx_hal::image::{
    Access, Extent, Kind, Layout, Offset, SubresourceLayers, SubresourceRange, Tiling, Usage,
    ViewCapabilities, ViewKind,
};
use gfx_hal::memory::{Dependencies, Properties, Segment};
use gfx_hal::pool::CommandPool;
use gfx_hal::pso::PipelineStage;
use gfx_hal::queue::CommandQueue;
use gfx_hal::{buffer, Backend};
use nalgebra_glm::TVec2;
use std::mem::ManuallyDrop;

#[derive(Debug)]
pub enum ImageResourceError {
    InvalidCopyRange,
}

pub struct ImageResource<B: Backend> {
    render_bundle: RenderBundleCommon<B>,
    image: ManuallyDrop<B::Image>,
    image_view: ManuallyDrop<B::ImageView>,
    image_memory: ManuallyDrop<B::Memory>,
    color_range: SubresourceRange,
    format: Format,
    size: TVec2<u32>,
}

impl<B: Backend> ImageResource<B> {
    pub fn new(
        render_bundle: &RenderBundleCommon<B>,
        size: TVec2<u32>,
        format: Format,
        usage: Usage,
        color_range: SubresourceRange,
    ) -> ImageResource<B> {
        let kind = Kind::D2(size.x, size.y, 1, 1);
        let device = render_bundle.device();
        let mut image = unsafe {
            device.create_image(
                kind,
                1,
                format,
                Tiling::Optimal,
                usage,
                ViewCapabilities::empty(),
            )
        }
        .unwrap();
        let image_req = unsafe { device.get_image_requirements(&image) };
        let device_type = find_memory_type(
            render_bundle.memory_types(),
            &image_req,
            Properties::DEVICE_LOCAL,
        );
        let image_memory = unsafe { device.allocate_memory(device_type, image_req.size) }.unwrap();
        unsafe { device.bind_image_memory(&image_memory, 0, &mut image) }.unwrap();
        let image_view = unsafe {
            device.create_image_view(
                &image,
                ViewKind::D2,
                format,
                Swizzle::NO,
                color_range.clone(),
            )
        }
        .unwrap();

        ImageResource {
            render_bundle: render_bundle.clone(),
            image: ManuallyDrop::new(image),
            image_view: ManuallyDrop::new(image_view),
            image_memory: ManuallyDrop::new(image_memory),
            color_range,
            format,
            size,
        }
    }

    pub fn new_for_texture(
        render_bundle: &RenderBundleCommon<B>,
        size: TVec2<u32>,
    ) -> ImageResource<B> {
        ImageResource::new(
            render_bundle,
            size,
            Format::Rgba8Srgb,
            Usage::TRANSFER_DST | Usage::SAMPLED,
            SubresourceRange {
                aspects: Aspects::COLOR,
                ..Default::default()
            },
        )
    }

    pub fn new_for_depth(
        render_bundle: &RenderBundleCommon<B>,
        size: TVec2<u32>,
    ) -> ImageResource<B> {
        ImageResource::new(
            render_bundle,
            size,
            Format::D32Sfloat,
            Usage::DEPTH_STENCIL_ATTACHMENT,
            SubresourceRange {
                aspects: Aspects::DEPTH,
                ..Default::default()
            },
        )
    }

    pub fn copy(
        &mut self,
        image_raw: &Image,
        offset: &TVec2<u32>,
    ) -> Result<(), ImageResourceError> {
        if image_raw.size().x + offset.x > self.size.x
            || image_raw.size().y + offset.y > self.size.y
        {
            return Err(ImageResourceError::InvalidCopyRange);
        }

        let image_stride = 4u32;
        let non_coherent_alignment = self.render_bundle.limits().non_coherent_atom_size as u64;
        let row_alignment_mask = self
            .render_bundle
            .limits()
            .optimal_buffer_copy_pitch_alignment as u32
            - 1;
        let row_pitch =
            (image_raw.size().x * image_stride + row_alignment_mask) & !row_alignment_mask;
        let upload_size = (image_raw.size().y * row_pitch) as u64;
        let padded_upload_size = ((upload_size + non_coherent_alignment - 1)
            / non_coherent_alignment)
            * non_coherent_alignment;

        let mut image_upload_buffer = unsafe {
            self.render_bundle
                .device()
                .create_buffer(padded_upload_size, buffer::Usage::TRANSFER_SRC)
        }
        .unwrap();
        let image_upload_buffer_req = unsafe {
            self.render_bundle
                .device()
                .get_buffer_requirements(&image_upload_buffer)
        };
        let memory_type = find_memory_type(
            self.render_bundle.memory_types(),
            &image_upload_buffer_req,
            Properties::CPU_VISIBLE,
        );
        let image_upload_memory = unsafe {
            let memory = self
                .render_bundle
                .device()
                .allocate_memory(memory_type, image_upload_buffer_req.size)
                .unwrap();
            self.render_bundle
                .device()
                .bind_buffer_memory(&memory, 0, &mut image_upload_buffer)
                .unwrap();
            let mapping = self
                .render_bundle
                .device()
                .map_memory(&memory, Segment::ALL)
                .unwrap();
            for y in 0..image_raw.size().y as usize {
                let row = image_raw.row(y);
                std::ptr::copy_nonoverlapping(
                    row.as_ptr(),
                    mapping.offset(y as isize * row_pitch as isize),
                    (image_raw.size().x * image_stride) as usize,
                );
            }
            self.render_bundle
                .device()
                .flush_mapped_memory_ranges(std::iter::once((&memory, Segment::ALL)))
                .unwrap();
            self.render_bundle.device().unmap_memory(&memory);
            memory
        };

        // copy buffer to texture
        let copy_fence = self
            .render_bundle
            .device()
            .create_fence(false)
            .expect("Could not create fence");
        unsafe {
            let mut cmd_buffer = self
                .render_bundle
                .command_pool_mut()
                .allocate_one(Level::Primary);
            cmd_buffer.begin_primary(CommandBufferFlags::ONE_TIME_SUBMIT);

            let image_barrier = gfx_hal::memory::Barrier::Image {
                states: (Access::empty(), Layout::Undefined)
                    ..(Access::TRANSFER_WRITE, Layout::TransferDstOptimal),
                target: &*self.image,
                families: None,
                range: self.color_range.clone(),
            };

            cmd_buffer.pipeline_barrier(
                PipelineStage::TOP_OF_PIPE..PipelineStage::TRANSFER,
                Dependencies::empty(),
                &[image_barrier],
            );

            cmd_buffer.copy_buffer_to_image(
                &image_upload_buffer,
                &self.image,
                gfx_hal::image::Layout::TransferDstOptimal,
                &[gfx_hal::command::BufferImageCopy {
                    buffer_offset: 0,
                    buffer_width: row_pitch / (image_stride as u32),
                    buffer_height: image_raw.size().y,
                    image_layers: SubresourceLayers {
                        aspects: Aspects::COLOR,
                        level: 0,
                        layers: 0..1,
                    },
                    image_offset: Offset {
                        x: offset.x as i32,
                        y: offset.y as i32,
                        z: 0,
                    },
                    image_extent: Extent {
                        width: image_raw.size().x,
                        height: image_raw.size().y,
                        depth: 1,
                    },
                }],
            );

            let image_barrier = gfx_hal::memory::Barrier::Image {
                states: (Access::TRANSFER_WRITE, Layout::TransferDstOptimal)
                    ..(Access::SHADER_READ, Layout::ShaderReadOnlyOptimal),
                target: &*self.image,
                families: None,
                range: self.color_range.clone(),
            };
            cmd_buffer.pipeline_barrier(
                PipelineStage::TRANSFER..PipelineStage::FRAGMENT_SHADER,
                Dependencies::empty(),
                &[image_barrier],
            );

            cmd_buffer.finish();

            self.render_bundle
                .primary_command_queue_mut()
                .submit_without_semaphores(Some(&cmd_buffer), Some(&copy_fence));

            self.render_bundle
                .device()
                .wait_for_fence(&copy_fence, !0)
                .expect("Can't wait for fence");

            self.render_bundle.device().destroy_fence(copy_fence);
            self.render_bundle.device().free_memory(image_upload_memory);
            self.render_bundle
                .device()
                .destroy_buffer(image_upload_buffer);

            self.render_bundle.command_pool_mut().free(vec![cmd_buffer]);
        }
        Ok(())
    }

    pub fn image_view(&self) -> &B::ImageView {
        &self.image_view
    }

    pub fn format(&self) -> &Format {
        &self.format
    }
}

impl<B: Backend> Drop for ImageResource<B> {
    fn drop(&mut self) {
        unsafe {
            self.render_bundle
                .device()
                .destroy_image_view(ManuallyDrop::into_inner(std::ptr::read(&self.image_view)));
            self.render_bundle
                .device()
                .destroy_image(ManuallyDrop::into_inner(std::ptr::read(&self.image)));
            self.render_bundle
                .device()
                .free_memory(ManuallyDrop::into_inner(std::ptr::read(&self.image_memory)));
        }
    }
}
