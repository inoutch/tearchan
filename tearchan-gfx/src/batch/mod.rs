use crate::batch::object::{BatchObject, BatchObjectCommand};
use crate::batch::types::{BatchAttributeIndex, BatchTypeArray, BatchTypeTransform};
use std::collections::hash_map::RandomState;
use std::collections::{HashMap, HashSet};
use std::sync::mpsc::{channel, Receiver, Sender};
use tearchan_util::btree::DuplicatableBTreeMap;
use tearchan_util::id_manager::{IdGenerator, IdManager};

const DEFAULT_ORDER: i32 = 0;

pub type BatchObjectId = u64;

pub mod batch2d;
pub mod batch3d;
pub mod batch_billboard;
pub mod batch_line;
pub mod buffer;
pub mod object;
pub mod types;
pub mod v2;

pub enum BatchProviderCommand<'a> {
    Add {
        id: BatchObjectId,
        data: &'a Vec<BatchTypeArray>,
        order: &'a Option<i32>,
    },
    Remove {
        id: BatchObjectId,
    },
    Replace {
        id: BatchObjectId,
        attribute: BatchAttributeIndex,
        data: &'a BatchTypeArray,
    },
}

pub trait BatchProvider {
    type Device;
    type Queue;
    type Encoder;

    fn run(
        &mut self,
        device: &Self::Device,
        queue: &Self::Queue,
        encoder: &mut Option<&mut Self::Encoder>,
        command: BatchProviderCommand,
    );

    fn sort(&mut self, ids: Vec<u64>) -> HashSet<u32, RandomState>;

    fn flush(&mut self, queue: &Self::Queue, manager: &mut BatchObjectManager);
}

pub struct BatchCommandBuffer {
    id_generator: IdGenerator<BatchObjectId>,
    command_sender: Sender<BatchObjectCommand>,
}

impl BatchCommandBuffer {
    pub fn add(&mut self, data: Vec<BatchTypeArray>, order: Option<i32>) -> BatchObjectId {
        let id = self.id_generator.gen();
        self.command_sender
            .send(BatchObjectCommand::Add { id, data, order })
            .unwrap();
        id
    }

    pub fn remove(&mut self, id: BatchObjectId) {
        self.command_sender
            .send(BatchObjectCommand::Remove { id })
            .unwrap();
    }

    pub fn replace(
        &mut self,
        id: BatchObjectId,
        attribute: BatchAttributeIndex,
        data: BatchTypeArray,
    ) {
        self.command_sender
            .send(BatchObjectCommand::Replace {
                id,
                attribute,
                data,
            })
            .unwrap();
    }

    pub fn transform(
        &mut self,
        id: BatchObjectId,
        attribute: BatchAttributeIndex,
        transform: BatchTypeTransform,
    ) {
        self.command_sender
            .send(BatchObjectCommand::Transform {
                id,
                attribute,
                transform,
            })
            .unwrap();
    }
}

pub struct Batch<T>
where
    T: BatchProvider,
{
    id_manager: IdManager<BatchObjectId>,
    command_receiver: Receiver<BatchObjectCommand>,
    command_sender: Sender<BatchObjectCommand>,
    batch_object_manager: BatchObjectManager,
    provider: T,
}

impl<T> Batch<T>
where
    T: BatchProvider,
{
    pub fn new(provider: T) -> Batch<T> {
        let (tx, rx) = channel();
        Batch {
            id_manager: IdManager::new(0, |id| *id + 1),
            command_receiver: rx,
            command_sender: tx,
            batch_object_manager: BatchObjectManager::default(),
            provider,
        }
    }
}

impl<T> Batch<T>
where
    T: BatchProvider,
{
    pub fn create_command_buffer(&mut self) -> BatchCommandBuffer {
        BatchCommandBuffer {
            id_generator: self.id_manager.create_generator(),
            command_sender: Sender::clone(&self.command_sender),
        }
    }

    pub fn flush(
        &mut self,
        device: &T::Device,
        queue: &T::Queue,
        encoder: &mut Option<&mut T::Encoder>,
    ) {
        let mut need_sort_all = false;
        while let Ok(command) = self.command_receiver.try_recv() {
            if let Some(provider_command) = match &command {
                BatchObjectCommand::Add { id, data, order } => {
                    if order.is_some() {
                        need_sort_all = true;
                    }
                    Some(BatchProviderCommand::Add {
                        id: *id,
                        data,
                        order,
                    })
                }
                BatchObjectCommand::Remove { id } => Some(BatchProviderCommand::Remove { id: *id }),
                BatchObjectCommand::Replace {
                    id,
                    attribute,
                    data,
                } => Some(BatchProviderCommand::Replace {
                    id: *id,
                    attribute: *attribute,
                    data,
                }),
                _ => None,
            } {
                self.provider.run(device, queue, encoder, provider_command);
            }
            self.batch_object_manager.run(command);
        }

        if need_sort_all {
            let attributes = self
                .provider
                .sort(self.batch_object_manager.create_sorted_ids());
            for attribute in attributes {
                self.batch_object_manager
                    .run(BatchObjectCommand::Refresh { attribute });
            }
        }

        self.provider.flush(queue, &mut self.batch_object_manager);
    }

    pub fn provider(&self) -> &T {
        &self.provider
    }
}

#[derive(Default)]
pub struct BatchObjectManager {
    objects: HashMap<BatchObjectId, BatchObject>,
    sorted_object_ids: DuplicatableBTreeMap<i32, BatchObjectId>,
    changed_object_attributes: HashMap<BatchObjectId, HashSet<BatchAttributeIndex>>,
}

impl BatchObjectManager {
    pub fn flush<F>(&mut self, mut callback: F)
    where
        F: FnMut(&mut BatchObject, u32),
    {
        for (id, attributes) in &mut self.changed_object_attributes {
            let object = match self.objects.get_mut(id) {
                Some(object) => object,
                None => continue,
            };
            for attribute in attributes.iter() {
                callback(object, *attribute);
            }
        }
        self.changed_object_attributes.clear();
    }

    fn run(&mut self, command: BatchObjectCommand) {
        match command {
            BatchObjectCommand::Add { id, data, order } => {
                self.command_add(id, data, order);
            }
            BatchObjectCommand::Remove { id } => {
                self.command_remove(id);
            }
            BatchObjectCommand::Transform {
                id,
                attribute,
                transform,
            } => {
                self.command_transform(id, attribute, transform);
            }
            BatchObjectCommand::Replace {
                id,
                data,
                attribute,
            } => {
                self.command_replace(id, data, attribute);
            }
            BatchObjectCommand::Refresh { attribute } => {
                self.command_refresh(attribute);
            }
        }
    }

    fn command_add(&mut self, id: BatchObjectId, data: Vec<BatchTypeArray>, order: Option<i32>) {
        let order = order.unwrap_or(DEFAULT_ORDER);
        let transforms = vec![BatchTypeTransform::None; data.len()];
        let mut set = HashSet::with_capacity(data.len());
        for i in 0u32..data.len() as u32 {
            set.insert(i);
        }

        self.objects
            .insert(id, BatchObject::new(id, data, transforms, order));
        self.sorted_object_ids.push_back(order, id);
        self.changed_object_attributes.insert(id, set);
    }

    fn command_remove(&mut self, id: BatchObjectId) {
        self.changed_object_attributes.remove(&id);
        if let Some(object) = self.objects.remove(&id) {
            self.sorted_object_ids.remove(&object.order(), &id);
        }
    }

    fn command_transform(
        &mut self,
        id: BatchObjectId,
        attribute: BatchAttributeIndex,
        transform: BatchTypeTransform,
    ) {
        let object = match self.objects.get_mut(&id) {
            Some(object) => object,
            None => return,
        };
        object.set_transform(attribute, transform);

        if let Some(set) = self.changed_object_attributes.get_mut(&id) {
            set.insert(attribute);
        } else {
            let mut set = HashSet::new();
            set.insert(attribute);
            self.changed_object_attributes.insert(id, set);
        }
    }

    fn command_replace(
        &mut self,
        id: BatchObjectId,
        data: BatchTypeArray,
        attribute: BatchAttributeIndex,
    ) {
        let object = match self.objects.get_mut(&id) {
            Some(object) => object,
            None => return,
        };
        object.set_data(attribute, data);

        self.refresh_attribute(id, attribute);
    }

    fn command_refresh(&mut self, attribute: BatchAttributeIndex) {
        let ids = self.objects.keys().into_iter().copied().collect::<Vec<_>>();
        for id in ids {
            self.refresh_attribute(id, attribute);
        }
    }

    fn refresh_attribute(&mut self, id: BatchObjectId, attribute: BatchAttributeIndex) {
        match self.changed_object_attributes.get_mut(&id) {
            None => {
                let mut set = HashSet::new();
                set.insert(attribute);
                self.changed_object_attributes.insert(id, set);
            }
            Some(set) => {
                set.insert(attribute);
            }
        }
    }

    fn create_sorted_ids(&self) -> Vec<BatchObjectId> {
        self.sorted_object_ids
            .iter()
            .map(|(_, ids)| ids)
            .flatten()
            .copied()
            .collect()
    }
}
