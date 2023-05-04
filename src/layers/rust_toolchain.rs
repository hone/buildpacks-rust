use crate::{util, RustBuildpack};
use libcnb::{
    build::BuildContext,
    data::layer_content_metadata::LayerTypes,
    generic::GenericMetadata,
    layer::{ExistingLayerStrategy, Layer, LayerData, LayerResult, LayerResultBuilder},
    Buildpack,
};
use libherokubuildpack::log::{log_header, log_info};
use std::{path::Path, process::Command};

const RUST_URL: &str =
    "https://static.rust-lang.org/dist/rust-1.69.0-x86_64-unknown-linux-gnu.tar.gz";

pub struct RustToolchain;

impl Layer for RustToolchain {
    type Buildpack = RustBuildpack;
    type Metadata = GenericMetadata;

    fn types(&self) -> LayerTypes {
        LayerTypes {
            launch: false,
            build: true,
            cache: true,
        }
    }

    fn create(
        &self,
        _context: &BuildContext<Self::Buildpack>,
        layer_path: &Path,
    ) -> Result<LayerResult<Self::Metadata>, <Self::Buildpack as Buildpack>::Error> {
        log_header("Setting up the Rust Toolchain");

        log_info("Downloading tarball");
        let mut rust_tarball = tempfile::tempfile()?;
        let size = util::download(RUST_URL, &mut rust_tarball)?;
        log_info(format!("File size: {size}"));

        log_info("Extracting tarball");
        let tmpdir = tempfile::tempdir()?;
        util::extract_tar_gz(&mut rust_tarball, tmpdir.path())?;

        log_info("Installing...");
        Command::new("./install.sh")
            .current_dir(tmpdir.path().join("rust-1.69.0-x86_64-unknown-linux-gnu"))
            .arg(format!("--destdir={}", layer_path.display()))
            .arg("--prefix=/")
            .spawn()
            .and_then(|mut child| child.wait())?;

        LayerResultBuilder::new(GenericMetadata::default()).build()
    }

    fn existing_layer_strategy(
        &self,
        _context: &BuildContext<Self::Buildpack>,
        _layer_data: &LayerData<Self::Metadata>,
    ) -> Result<ExistingLayerStrategy, <Self::Buildpack as Buildpack>::Error> {
        log_header("Re-use existing Rust Toolchain");
        Ok(ExistingLayerStrategy::Keep)
    }
}
