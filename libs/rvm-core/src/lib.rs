#![feature(map_try_insert)]

mod storage;
mod ty;
mod r#ref;

pub use r#ref::Ref;
pub use ty::*;
pub use storage::*;
use tracing::Level;
use tracing_subscriber::filter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

static mut INITIALIZED: bool = false;

pub fn init() {
	if !unsafe {
		let initialized = INITIALIZED;
		INITIALIZED = true;
		initialized
	} {
		let filter = filter::Targets::new()
			.with_default(Level::ERROR)
			.with_target("gc", Level::INFO)
			.with_target("exec", Level::INFO);

		tracing_subscriber::registry()
			.with(tracing_subscriber::fmt::layer())
			.with(filter)
			.init();
	}
}
