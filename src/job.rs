/// A Job Trait, taken from the `cron-job` crate as they did a pretty damn good
/// job with some minor modifications
#[async_trait::async_trait]
pub trait Job: Send + Sync + 'static {
	async fn run(&mut self);
}
