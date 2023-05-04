use crate::RustBuildpack;
use libcnb::{
    build::BuildContext,
    data::layer_content_metadata::LayerTypes,
    generic::GenericMetadata,
    layer::{Layer, LayerResult, LayerResultBuilder},
    Buildpack, Env,
};
use libherokubuildpack::log::log_header;
use std::{path::Path, process::Command};

pub struct Target {
    pub env: Env,
}

impl Layer for Target {
    type Buildpack = RustBuildpack;
    type Metadata = GenericMetadata;

    fn types(&self) -> LayerTypes {
        LayerTypes {
            launch: true,
            build: false,
            cache: true,
        }
    }

    fn create(
        &self,
        context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, <Self::Buildpack as Buildpack>::Error> {
        log_header("Running Cargo");

        Command::new("cargo")
            .arg("build")
            .arg("--release")
            .arg(format!("--target-dir={}", layer_path.display()))
            .current_dir(&context.app_dir)
            .envs(&self.env)
            .spawn()
            .and_then(|mut child| child.wait())?;

        LayerResultBuilder::new(GenericMetadata::default()).build()
    }
}
