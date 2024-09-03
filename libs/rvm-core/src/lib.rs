#![feature(map_try_insert)]
#![feature(associated_type_defaults)]

use std::sync::Once;

use tracing::Level;
use tracing_subscriber::filter;
use tracing_subscriber::layer::SubscriberExt;

pub use storage::*;
pub use ty::*;

mod storage;
mod ty;

static START: Once = Once::new();

pub fn init() {
	START.call_once(|| {
		let filter = filter::Targets::new()
			.with_default(Level::TRACE)
			.with_target("exe", Level::DEBUG)
			.with_target("gc", Level::INFO)
			.with_target("exec", Level::INFO);
		let layered = tracing_subscriber::registry()
			.with(tracing_subscriber::fmt::layer())
			.with(filter);

		tracing::subscriber::set_global_default(layered).unwrap();
	});
}
