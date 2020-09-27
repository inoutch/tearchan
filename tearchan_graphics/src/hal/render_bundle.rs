use crate::display_size::DisplaySize;
use gfx_hal::adapter::{MemoryProperties, MemoryType};
use gfx_hal::queue::{QueueFamilyId, QueueGroup};
use gfx_hal::{Backend, Limits};
use std::cell::{Ref, RefMut};
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::rc::Rc;
use tearchan_utility::shared::Shared;

pub struct RenderBundleCommon<B: Backend> {
    device: Rc<B::Device>,
    queue_group: Shared<QueueGroup<B>>,
    display_size: Shared<DisplaySize>,
    memory_properties: Rc<MemoryProperties>,
    limits: Rc<Limits>,
    // Manually resources
    command_pool: Shared<ManuallyDrop<B::CommandPool>>,
}

impl<B: Backend> RenderBundleCommon<B> {
    pub fn new(
        device: Rc<B::Device>,
        queue_group: Shared<QueueGroup<B>>,
        display_size: Shared<DisplaySize>,
        memory_properties: Rc<MemoryProperties>,
        limits: Rc<Limits>,
        command_pool: Shared<ManuallyDrop<B::CommandPool>>,
    ) -> RenderBundleCommon<B> {
        RenderBundleCommon {
            device,
            queue_group,
            command_pool,
            display_size,
            memory_properties,
            limits,
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn clone(&self) -> RenderBundleCommon<B> {
        RenderBundleCommon {
            device: Rc::clone(&self.device),
            queue_group: Shared::clone(&self.queue_group),
            command_pool: Shared::clone(&self.command_pool),
            display_size: Shared::clone(&self.display_size),
            memory_properties: Rc::clone(&self.memory_properties),
            limits: Rc::clone(&self.limits),
        }
    }

    pub fn primary_command_queue(&self) -> Ref<B::CommandQueue> {
        Ref::map(self.queue_group.borrow(), |queue_group| {
            &queue_group.queues[0]
        })
    }

    pub fn primary_command_queue_mut(&mut self) -> RefMut<B::CommandQueue> {
        RefMut::map(self.queue_group.borrow_mut(), |queue_group| {
            &mut queue_group.queues[0]
        })
    }

    pub fn command_pool(&self) -> Ref<ManuallyDrop<B::CommandPool>> {
        self.command_pool.borrow()
    }

    pub fn command_pool_mut(&mut self) -> RefMut<ManuallyDrop<B::CommandPool>> {
        self.command_pool.borrow_mut()
    }

    pub fn device(&self) -> &B::Device {
        self.device.deref()
    }

    pub fn display_size(&self) -> Ref<DisplaySize> {
        self.display_size.borrow()
    }

    pub fn memory_types(&self) -> &Vec<MemoryType> {
        &self.memory_properties.memory_types
    }

    pub fn limits(&self) -> &Limits {
        &self.limits
    }

    pub fn queue_family(&self) -> QueueFamilyId {
        self.queue_group.borrow().family
    }
}
