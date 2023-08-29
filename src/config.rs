use std::str::FromStr;

use tokio::sync::Mutex;

use crate::{
	error::BuildError,
	Job,
};

#[derive(Debug, Default)]
pub struct SchduleBuilder {
	schedule: Option<cron::Schedule>,
	active:   Option<bool>,
}

impl SchduleBuilder {
	pub fn new() -> Self {
		SchduleBuilder {
			..Default::default()
		}
	}

	pub fn with_schedule(
		&mut self,
		schedule: cron::Schedule,
	) -> &mut Self {
		self.schedule = Some(schedule);
		self
	}

	pub fn activate(
		&mut self,
		activate: bool,
	) -> &mut Self {
		self.active = Some(activate);
		self
	}

	pub fn with_schedule_str<'a>(
		&mut self,
		schedule: impl Into<&'a str>,
	) -> Result<&mut Self, cron::error::Error> {
		self.schedule = Some(cron::Schedule::from_str(schedule.into())?);
		Ok(self)
	}

	pub fn build<T>(
		self,
		callback: T,
	) -> Result<Schedule, BuildError>
	where
		T: Job,
	{
		let schedule = match self.schedule {
			Some(sch) => sch,
			None => return Err(BuildError::MissingSchedule),
		};

		let active = match self.active {
			Some(act) => act,
			None => return Err(BuildError::MissingActivation),
		};

		Ok(Schedule {
			schedule,
			active,
			callback: Mutex::new(Box::new(callback)),
		})
	}
}

pub struct Schedule {
	schedule: cron::Schedule,
	active:   bool,
	callback: Mutex<Box<dyn Job>>,
}

impl Schedule {
	pub fn builder() -> SchduleBuilder {
		SchduleBuilder::new()
	}

	pub fn get_schedule(&self) -> &cron::Schedule {
		&self.schedule
	}

	pub fn is_active(&self) -> bool {
		self.active
	}

	pub(crate) fn get_next_invocation(&self) -> Option<chrono::DateTime<chrono::Utc>> {
		let time = chrono::Utc::now();
		self.schedule.after(&time).next()
	}

	pub async fn run(&self) {
		let l = self.callback.try_lock().ok();
		if let Some(mut l) = l {
			l.run().await;
		}
	}
}
