#![feature(map_try_insert)]
#![feature(associated_type_defaults)]

use std::sync::Once;

use tracing::Level;
use tracing_subscriber::filter;
use tracing_subscriber::layer::SubscriberExt;

pub use storage::*;
pub use ty::*;
pub use utils::*;

mod storage;
mod ty;
mod utils;

static START: Once = Once::new();

pub fn init() {
	START.call_once(|| {
		let filter = filter::Targets::new()
			.with_default(Level::TRACE)
			.with_target("rvm_gc", Level::INFO)
			.with_target("rvm_stack", Level::DEBUG)
			.with_target("exe", Level::TRACE)
			.with_target("exec", Level::TRACE);
		let layered = tracing_subscriber::registry()
			.with(tracing_subscriber::fmt::layer())
			.with(filter);

		tracing::subscriber::set_global_default(layered).unwrap();
	});
}
