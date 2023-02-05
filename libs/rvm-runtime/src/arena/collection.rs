use std::thread::{current, sleep, spawn};
use mmtk::util::opaque_pointer::*;
use mmtk::vm::Collection;
use mmtk::vm::GCThreadContext;
use mmtk::Mutator;
use mmtk::MutatorContext;
use mmtk::util::Address;
use tracing::{debug, info};
use crate::arena::{Arena, SINGLETON};

pub struct VMCollection {
}

impl Collection<Arena> for VMCollection {
    fn stop_all_mutators<F>(_tls: VMWorkerThread, _mutator_visitor: F)
        where
            F: FnMut(&'static mut Mutator<Arena>),
    {
        unimplemented!()
    }

    fn resume_mutators(_tls: VMWorkerThread) {
        unimplemented!()
    }

    fn block_for_gc(_tls: VMMutatorThread) {
        debug!("Blocking gc {:?}", _tls);

        // panic!("block_for_gc is not implemented")
    }

    fn spawn_gc_thread(tls: VMThread, ctx: GCThreadContext<Arena>) {
        debug!("Spawning GC Thread {:?}", tls);

        spawn(move || unsafe {
            let id = current().id();
            let thread = VMWorkerThread(VMThread(OpaquePointer::from_address(Address::from_usize(id.as_u64().get() as usize))));
            match ctx {
                GCThreadContext::Controller(mut controller) => {
                    controller.run(thread);
                }
                GCThreadContext::Worker(mut worker) => {
                    worker.run(thread, &SINGLETON);
                }
            };
        });
    }

    fn prepare_mutator<T: MutatorContext<Arena>>(
        _tls_w: VMWorkerThread,
        _tls_m: VMMutatorThread,
        _mutator: &T,
    ) {
        unimplemented!()
    }
}