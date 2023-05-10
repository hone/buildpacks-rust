use crate::{util, RustBuildpack};
use libcnb::{
    build::BuildContext,
    data::layer_content_metadata::LayerTypes,
    generic::GenericMetadata,
    layer::{ExistingLayerStrategy, Layer, LayerData, LayerResult, LayerResultBuilder},
    Buildpack,
};
use libherokubuildpack::log::{log_header, log_info};
use rust_releases::{Channel, FetchResources, ReleaseIndex, RustDist};
use std::{path::Path, process::Command};

const ARCH: &str = "x86_64";
const VENDOR: &str = "unknown";
const OS: &str = "linux";
const ENV: &str = "gnu";

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
        let release_index = release_index();
        let release = release_index.most_recent().unwrap();
        let version = release.version();

        log_info(format!("Latest Stable: {}", release.version()));
        log_info("Downloading tarball...");
        let rust_long = format!("rust-{}-{}-{}-{}-{}", version, ARCH, VENDOR, OS, ENV);
        let mut rust_tarball = tempfile::tempfile()?;
        util::download(
            &format!("https://static.rust-lang.org/dist/{rust_long}.tar.gz",),
            &mut rust_tarball,
        )?;

        log_info("Extracting tarball...");
        let tmpdir = tempfile::tempdir()?;
        util::extract_tar_gz(&mut rust_tarball, tmpdir.path())?;

        log_info("Installing...");
        Command::new("./install.sh")
            .current_dir(tmpdir.path().join(rust_long))
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

fn release_index() -> ReleaseIndex {
    let source = RustDist::fetch_channel(Channel::Stable).unwrap();
    ReleaseIndex::from_source(source).unwrap()
}
