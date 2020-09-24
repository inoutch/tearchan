use crate::hal::render_bundle::RenderBundleCommon;
use gfx_hal::command::Level;
use gfx_hal::device::Device;
use gfx_hal::pool::{CommandPool, CommandPoolCreateFlags};
use gfx_hal::Backend;
use std::mem::ManuallyDrop;
use std::ops::Deref;

pub struct FrameResource<B: Backend> {
    render_bundle: RenderBundleCommon<B>,
    command_pool: ManuallyDrop<B::CommandPool>,
    command_buffer: B::CommandBuffer,
    submission_complete_fence: ManuallyDrop<B::Fence>,
    submission_complete_semaphore: ManuallyDrop<B::Semaphore>,
}

impl<B: Backend> FrameResource<B> {
    pub fn new(render_bundle: &RenderBundleCommon<B>) -> FrameResource<B> {
        let mut command_pool = unsafe {
            render_bundle.device().create_command_pool(
                render_bundle.queue_family(),
                CommandPoolCreateFlags::RESET_INDIVIDUAL,
            )
        }
        .expect("Can't create command pool");

        let command_buffer = unsafe { command_pool.allocate_one(Level::Primary) };

        let submission_complete_semaphore = render_bundle.device().create_semaphore().unwrap();

        let submission_complete_fence = render_bundle.device().create_fence(true).unwrap();

        FrameResource {
            render_bundle: render_bundle.clone(),
            command_pool: ManuallyDrop::new(command_pool),
            command_buffer,
            submission_complete_semaphore: ManuallyDrop::new(submission_complete_semaphore),
            submission_complete_fence: ManuallyDrop::new(submission_complete_fence),
        }
    }

    pub fn fence(&self) -> &B::Fence {
        self.submission_complete_fence.deref()
    }

    pub fn command_buffer(&self) -> &B::CommandBuffer {
        &self.command_buffer
    }

    pub fn command_buffer_mut(&mut self) -> &mut B::CommandBuffer {
        &mut self.command_buffer
    }

    pub fn submission_complete_fence(&self) -> &B::Fence {
        &self.submission_complete_fence
    }

    pub fn submission_complete_semaphore(&self) -> &B::Semaphore {
        &self.submission_complete_semaphore
    }
}

impl<B: Backend> Drop for FrameResource<B> {
    fn drop(&mut self) {
        unsafe {
            self.render_bundle
                .device()
                .destroy_command_pool(ManuallyDrop::into_inner(std::ptr::read(&self.command_pool)));

            self.render_bundle
                .device()
                .destroy_semaphore(ManuallyDrop::into_inner(std::ptr::read(
                    &self.submission_complete_semaphore,
                )));

            self.render_bundle
                .device()
                .destroy_fence(ManuallyDrop::into_inner(std::ptr::read(
                    &self.submission_complete_fence,
                )));
        }
    }
}
