use std::collections::VecDeque;
use std::sync::mpsc::{channel, Receiver, Sender};

#[derive(Default)]
pub struct ResourceSync {
    communicators: VecDeque<(Sender<()>, Receiver<()>)>,
}

impl ResourceSync {
    pub fn child(&mut self) -> ResourceSyncChild {
        let (sender0, receiver0) = channel();
        let (sender1, receiver1) = channel();
        self.communicators.push_back((sender0, receiver1));
        ResourceSyncChild {
            communicator: (sender1, receiver0),
        }
    }

    pub fn join(&mut self) {
        while let Some((sender, receiver)) = self.communicators.pop_front() {
            sender.send(()).unwrap();
            receiver.recv().unwrap();
        }
    }
}

pub struct ResourceSyncChild {
    communicator: (Sender<()>, Receiver<()>),
}

impl ResourceSyncChild {
    pub fn begin(&self) {
        self.communicator.1.recv().unwrap();
    }

    pub fn end(&self) {
        self.communicator.0.send(()).unwrap();
    }
}

#[cfg(test)]
mod test {
    use crate::component::group_sync::ComponentGroupSync;
    use crate::component::zip::ZipEntity1;
    use std::collections::VecDeque;
    use std::sync::mpsc::{channel, Receiver, Sender};
    use std::time::Duration;
    use tearchan_util::thread::ThreadPool;

    #[test]
    fn prevent_deadlock() {
        struct Blob0(Vec<i32>);
        struct Blob1(Vec<i32>);
        struct Blob2(Vec<i32>);

        // dep graph
        // 0 ---> 1
        //    1 -> 2
        //  2 ------> 0
        #[derive(Default)]
        struct ResourceSync {
            senders: VecDeque<(Sender<()>, Receiver<()>)>,
        }

        impl ResourceSync {
            fn child(&mut self) -> ResourceSyncChild {
                let (sender0, receiver0) = channel();
                let (sender1, receiver1) = channel();
                self.senders.push_back((sender0, receiver1));
                ResourceSyncChild {
                    receiver: (sender1, receiver0),
                }
            }

            fn join(&mut self) {
                while let Some((sender, receiver)) = self.senders.pop_front() {
                    sender.send(()).unwrap();
                    receiver.recv().unwrap();
                }
            }
        }

        struct ResourceSyncChild {
            receiver: (Sender<()>, Receiver<()>),
        }

        impl ResourceSyncChild {
            fn begin(&self) {
                self.receiver.1.recv().unwrap();
            }

            fn end(&self) {
                self.receiver.0.send(()).unwrap();
            }
        }

        let pool = ThreadPool::new(4);
        let mut rsync = ResourceSync::default();

        let mut cg0: ComponentGroupSync<Blob0> = ComponentGroupSync::default();
        let mut cg1: ComponentGroupSync<Blob1> = ComponentGroupSync::default();
        let mut cg2: ComponentGroupSync<Blob2> = ComponentGroupSync::default();

        cg0.write().get_mut().push(1, Blob0(vec![0]));
        cg1.write().get_mut().push(1, Blob1(vec![1]));
        cg2.write().get_mut().push(1, Blob2(vec![2]));

        let rsync_child = rsync.child();
        let cgr0 = cg0.read();
        let mut cgw0 = cg1.write();
        pool.execute(move || {
            std::thread::sleep(Duration::from_millis(100));
            rsync_child.begin();
            let cgr0 = &cgr0.get();
            let mut cgw0 = cgw0.get_mut();
            rsync_child.end();

            std::thread::sleep(Duration::from_millis(200));
            cgw0.iter_mut()
                .zip_entities_mut(&ZipEntity1::new(cgr0))
                .for_each(|(_entity_id, write, read)| {
                    write.0.append(&mut read.0.clone());
                });
        });

        let rsync_child = rsync.child();
        let cgr1 = cg1.read();
        let mut cgw2 = cg2.write();
        pool.execute(move || {
            rsync_child.begin();
            let cgr1 = &cgr1.get();
            let mut cgw2 = cgw2.get_mut();
            rsync_child.end();

            cgw2.iter_mut()
                .zip_entities_mut(&ZipEntity1::new(cgr1))
                .for_each(|(_entity_id, write, read)| {
                    write.0.append(&mut read.0.clone());
                });
        });

        let rsync_child = rsync.child();
        let cgr2 = cg2.read();
        let mut cgw0 = cg0.write();
        pool.execute(move || {
            rsync_child.begin();
            let cgr2 = &cgr2.get();
            let mut cgw2 = cgw0.get_mut();
            rsync_child.end();

            cgw2.iter_mut()
                .zip_entities_mut(&ZipEntity1::new(cgr2))
                .for_each(|(_entity_id, write, read)| {
                    write.0.append(&mut read.0.clone());
                });
        });

        rsync.join();
        pool.join();

        assert_eq!(cg0.read().get().get(1).unwrap().0, vec![0, 2, 1, 0]);
        assert_eq!(cg1.read().get().get(1).unwrap().0, vec![1, 0]);
        assert_eq!(cg2.read().get().get(1).unwrap().0, vec![2, 1, 0]);
    }
}
