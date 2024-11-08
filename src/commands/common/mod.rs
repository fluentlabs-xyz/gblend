use crate::error::Error;

pub trait Command {
    fn execute(self) -> Result<(), Error>;
}

pub mod deploy;
pub mod templates;
pub mod types;
