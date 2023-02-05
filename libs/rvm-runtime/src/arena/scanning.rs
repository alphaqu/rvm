use mmtk::util::opaque_pointer::*;
use mmtk::util::ObjectReference;
use mmtk::vm::EdgeVisitor;
use mmtk::vm::RootsWorkFactory;
use mmtk::vm::Scanning;
use mmtk::Mutator;
use mmtk::vm::edge_shape::SimpleEdge;
use crate::arena::Arena;

pub struct VMScanning {}

impl Scanning<Arena> for VMScanning {
    fn scan_thread_roots(_tls: VMWorkerThread, _factory: impl RootsWorkFactory<SimpleEdge>) {
        unimplemented!()
    }
    fn scan_thread_root(
        _tls: VMWorkerThread,
        _mutator: &'static mut Mutator<Arena>,
        _factory: impl RootsWorkFactory<SimpleEdge>,
    ) {
        unimplemented!()
    }
    fn scan_vm_specific_roots(_tls: VMWorkerThread, _factory: impl RootsWorkFactory<SimpleEdge>) {
        unimplemented!()
    }
    fn scan_object<EV: EdgeVisitor<SimpleEdge>>(
        _tls: VMWorkerThread,
        _object: ObjectReference,
        _edge_visitor: &mut EV,
    ) {
        unimplemented!()
    }
    fn notify_initial_thread_scan_complete(_partial_scan: bool, _tls: VMWorkerThread) {
        unimplemented!()
    }
    fn supports_return_barrier() -> bool {
        unimplemented!()
    }
    fn prepare_for_roots_re_scanning() {
        unimplemented!()
    }
}