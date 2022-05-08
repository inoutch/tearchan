use crate::component::group::ComponentGroup;
use crate::component::group_sync::{ComponentGroupSyncReader, ComponentGroupSyncWriter};
use tearchan_util::thread::ThreadPool;

pub trait SystemJob<TW, TR>
where
    TW: Sync + Send + 'static,
    TR: Sync + Send + 'static,
{
    fn run(write: &mut ComponentGroup<TW>, read: &ComponentGroup<TR>);

    fn run_async(
        thread_pool: &ThreadPool,
        mut write: ComponentGroupSyncWriter<TW>,
        read: ComponentGroupSyncReader<TR>,
    ) {
        thread_pool.execute(move || {
            Self::run(&mut write.get_mut(), &read.get());
        });
    }
}

#[cfg(test)]
mod test {
    use crate::component::group::ComponentGroup;
    use crate::component::group_sync::ComponentGroupSync;
    use crate::component::zip::ZipEntity1;
    use crate::system::SystemJob;
    use tearchan_util::thread::ThreadPool;

    struct CustomSystem;

    struct Component1 {
        pub value: u32,
    }

    struct Component2 {
        pub value: u32,
    }

    impl SystemJob<Component1, Component2> for CustomSystem {
        fn run(write: &mut ComponentGroup<Component1>, read: &ComponentGroup<Component2>) {
            write
                .iter_mut()
                .zip_entities_mut(&ZipEntity1::new(read))
                .for_each(|(_entity_id, write, read)| {
                    write.value += read.value;
                });
        }
    }

    #[test]
    fn test_on_thread_pool() {
        let thread_pool = ThreadPool::new(4);
        let mut group1 = ComponentGroupSync::default();
        let mut group2 = ComponentGroupSync::default();
        {
            let mut writer1 = group1.write();
            let mut writer2 = group2.write();
            writer1.get_mut().push(0, Component1 { value: 0 });
            writer2.get_mut().push(0, Component2 { value: 2 });

            writer1.get_mut().push(1, Component1 { value: 1 });
            writer2.get_mut().push(1, Component2 { value: 3 });
        }

        {
            let r = group1.read();
            assert_eq!(r.get().get(0).unwrap().value, 0);
            assert_eq!(r.get().get(1).unwrap().value, 1);
        }

        CustomSystem::run_async(&thread_pool, group1.write(), group2.read());

        thread_pool.join();

        {
            let r = group1.read();
            assert_eq!(r.get().get(0).unwrap().value, 2);
            assert_eq!(r.get().get(1).unwrap().value, 4);
        }
    }
}
