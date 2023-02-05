use std::sync::{Arc, Condvar, Mutex};
use std::thread::Builder;

pub struct ThreadManager {
	list: Vec<VMThread>,
}

thread_local! {
	static DATA: Arc<VMThreadHandle> = Arc::new(VMThreadHandle::default());
}


pub struct VMThread {
	data: Arc<VMThreadHandle>,
}

impl VMThread {
	pub fn new<F>(builder: Builder, func: F) -> Self
	where
		F: FnOnce() -> (),
		F: Send + 'static,
	{
        todo!()
	}

    /// Blocks the thread until the unblock method has been called.
    ///
    /// This method itself will block until the thread has been blocked from executing further.
	pub fn block(&self) {
        todo!()
		//*self.data.should_stop.lock().unwrap() = true;
		//let guard = self.data.stopped.lock().unwrap();
		//let _guard = self
		//	.data
		//	.stopped_cond
		//	.wait_while(guard, |pending| !*pending)
		//	.unwrap();
	}

    /// Unblocks the threads from a previous block()
    pub fn unblock(&self) {
        todo!();
    }
}

/// Checks if the thread should be blocked and notifies the main thread about its blockage.
pub fn yield_thread() {
	DATA.with(|v| {
		let arc = v.clone();
		let guard = arc.should_stop.lock().unwrap();
		if *guard == true {
			*arc.stopped.lock().unwrap() = true;
			arc.stopped_cond.notify_one();
			let _guard = arc
				.should_stop_cond
				.wait_while(guard, |pending| *pending)
				.unwrap();
			*arc.stopped.lock().unwrap() = false;
			arc.stopped_cond.notify_one();
		}
	})
}

#[derive(Default)]
pub struct VMThreadHandle {
	should_stop: Atin,
	should_stop_cond: Condvar,
	stopped: Mutex<bool>,
	stopped_cond: Condvar,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn hi() {
		println!("hi");
	}
}
