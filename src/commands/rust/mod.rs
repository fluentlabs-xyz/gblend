mod build;
mod constants;
mod deploy;
mod init;

pub use self::{build::BuildArgs, deploy::DeployArgs, init::InitArgs};
use crate::error::Error;

pub struct RustCommand;

impl RustCommand {
    pub fn init(args: &InitArgs) -> Result<(), Error> {
        init::execute(args)
    }

    pub fn build(args: &BuildArgs) -> Result<(), Error> {
        build::execute(args)
    }

    pub async fn deploy(args: &DeployArgs, network: &str) -> Result<(), Error> {
        deploy::execute(args, network).await
    }
}
