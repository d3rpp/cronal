use std::{
	collections::HashMap,
	sync::Arc,
};

use chrono::Utc;
use tokio::sync::RwLock;

use crate::Schedule;

pub struct CronJob {
	/// The amount of milliseconds to wait between checking for jobs
	/// 
	/// This value is not guaranteed and is to be considered a timeout 
	/// for awaiting tokio doing its thing, for more info on why refer to
	/// [yield_now](tokio::task::yield_now)
	delay:     u64,
	/// I promise there is a reason for this.
	schedules: Arc<RwLock<HashMap<String, Arc<RwLock<Schedule>>>>>,
}

impl CronJob {
	pub fn new(delay: impl Into<u64>) -> Self {
		CronJob {
			delay:     delay.into(),
			schedules: Arc::new(RwLock::new(HashMap::new())),
		}
	}

	pub async fn start(&self) -> tokio::task::JoinHandle<()> {
		let sch = Arc::clone(&self.schedules);

		loop {
			// wait up until delay time, or less, if there are other tasks that should be
			// completed first, let them run and await them before running
			tokio::select! {
				_ = tokio::time::sleep(
					std::time::Duration::from_millis(self.delay)
				) => {},

				_ = tokio::task::yield_now() => {}
			}

			let lock = sch.read().await;

			let mut upcoming = vec![];

			for (id, sched) in lock.iter() {
				let sched_read = sched.read().await;
				upcoming.push((id.clone(), sched_read.get_next_invocation()));
			}

			for (i, upcoming) in upcoming.iter() {
				if let Some(dt) = upcoming {
					if dt.timestamp() <= Utc::now().timestamp() {
						let job_to_run = lock[i].clone();

						// the idea behind double layering an arc mutex is that if a job is deleted while it's running, 
						// it will be removed from the map but not deleted until this lock in the job here is released.
						tokio::spawn(async move { job_to_run.write().await.run().await });
					}
				}
			}
		}
	}

	/// returns the names of all currently active jobs
	pub async fn get_job_names(&self) -> Vec<String> {
		// welcome to rust, it has functional parasigms
		let sched_lock = self.schedules.read().await;

		let mut active_names = vec![];

		for (id, sched) in sched_lock.iter() {
			if sched.read().await.is_active() {
				active_names.push(id.clone());
			}
		}

		active_names
	}

	/// remove a job from the schedule, it will not run anymore and will not be
	/// taken into account on next invocation if the job was there and remove,
	/// this function will return true, otherwise the function will return
	/// false.
	pub async fn remove_job(
		&self,
		job_name: impl Into<String>,
	) -> bool {
		let mut lock = self.schedules.write().await;

		lock.remove(&job_name.into()).is_some()
	}

	/// will check if a job exist, by its job_name, if it does it will update
	/// the job with the given schedule item and return false, as it was an
	/// update.
	///
	/// If there is no job by its name, this function will return true because a
	/// new job was added to the schedule.
	pub async fn insert_job(
		&self,
		job_name: impl Into<String>,
		schedule: Schedule,
	) -> bool {
		let mut lock = self.schedules.write().await;

		lock.insert(job_name.into(), Arc::new(RwLock::new(schedule)))
			.is_none()
	}

	pub async fn has_job(&self, job_name: impl Into<String>) -> bool {
		let lock = self.schedules.read().await;

		lock.contains_key(&job_name.into())
	}
}
