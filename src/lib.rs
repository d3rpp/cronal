//! # Cronal
//!
//! An asynchronous Cron runtime designed to be continuously updated,
//! configuration is designed to be automatically updated as it runs,
//! heavily inspired by, and operates much the same to [cron-job](https://github.com/nambrosini/cron-job).

mod config;
pub use config::*;

mod job;
pub use job::*;

mod scheduler;
pub use scheduler::*;

pub mod error;

#[async_trait::async_trait]
impl<T> Job for T
where
	T: Send + Sync + 'static + Fn(),
{
	async fn run(&mut self) {
		self()
	}
}
