use thiserror::Error;

#[derive(Debug, Error)]
pub enum BuildError {
	#[error("a schedule was not specified")]
	MissingSchedule,

	#[error("the schedule was not specified as active or not")]
	MissingActivation,
}
