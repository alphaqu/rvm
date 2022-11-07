#![feature(map_try_insert)]

use std::sync::atomic::{AtomicBool, Ordering};

use tracing::Level;
use tracing_subscriber::filter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub use storage::*;

mod storage;

pub fn init() {
	static INITIALIZED: AtomicBool = AtomicBool::new(false);

	if INITIALIZED
		.compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
		.is_ok()
	{
		let filter = filter::Targets::new()
			.with_default(Level::TRACE)
			.with_target("gc", Level::INFO)
			.with_target("exec", Level::INFO);

		tracing_subscriber::registry()
			.with(tracing_subscriber::fmt::layer())
			.with(filter)
			.init();
	}
}
