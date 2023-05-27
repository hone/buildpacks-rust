use libcnb::{
    build::{BuildContext, BuildResult},
    data::build_plan::BuildPlanBuilder,
    detect::{DetectContext, DetectResult, DetectResultBuilder},
    generic::{GenericMetadata, GenericPlatform},
    Buildpack,
};
use std::io;
use thiserror::Error;

mod builds;
mod layers;
mod util;

#[derive(Debug, Error)]
pub enum BuildpackError {
    #[error("could not write file")]
    Io(#[from] io::Error),
    #[error("could not download file")]
    Download(#[from] util::DownloadError),
    #[error("Rust Toolchain PATH is not set")]
    RustToolchainPath,
    #[error("Unable to get Cargo Metadata")]
    CargoMetadata(#[from] cargo_metadata::Error),
}

impl From<BuildpackError> for libcnb::Error<BuildpackError> {
    fn from(e: BuildpackError) -> libcnb::Error<BuildpackError> {
        libcnb::Error::BuildpackError(e)
    }
}

pub struct RustBuildpack;

impl Buildpack for RustBuildpack {
    type Platform = GenericPlatform;
    type Metadata = GenericMetadata;
    type Error = BuildpackError;

    fn detect(&self, context: DetectContext<Self>) -> libcnb::Result<DetectResult, Self::Error> {
        if context.app_dir.join("Cargo.lock").exists() {
            let build_plan = BuildPlanBuilder::new().provides("rust").requires("rust");

            DetectResultBuilder::pass()
                .build_plan(build_plan.build())
                .build()
        } else {
            DetectResultBuilder::fail().build()
        }
    }

    fn build(&self, context: BuildContext<Self>) -> libcnb::Result<BuildResult, Self::Error> {
        builds::build(context)
    }
}
