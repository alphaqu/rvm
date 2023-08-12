#![feature(map_try_insert)]
#![feature(associated_type_defaults)]

mod r#ref;
mod storage;
mod ty;

pub use r#ref::*;
use std::sync::Once;
pub use storage::*;
use tracing::Level;
use tracing_subscriber::filter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
pub use ty::*;

static START: Once = Once::new();
pub fn init() {
	START.call_once(|| {
		let filter = filter::Targets::new()
			.with_default(Level::TRACE)
			.with_target("gc", Level::INFO)
			.with_target("exec", Level::INFO);
		let layered = tracing_subscriber::registry()
			.with(tracing_subscriber::fmt::layer())
			.with(filter);

		tracing::subscriber::set_global_default(layered).unwrap();
	});
}
