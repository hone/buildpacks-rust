use crate::{layers, BuildpackError, RustBuildpack};
use cargo_metadata::MetadataCommand;
use libcnb::{
    build::{BuildContext, BuildResult, BuildResultBuilder},
    data::{
        launch::{LaunchBuilder, ProcessBuilder, ProcessType},
        layer_name,
    },
    layer_env::Scope,
    Env,
};
use std::{fs, io, path::Path};

pub fn build(context: BuildContext<RustBuildpack>) -> libcnb::Result<BuildResult, BuildpackError> {
    let toolchain_layer =
        context.handle_layer(layer_name!("rust-toolchain"), layers::RustToolchain)?;
    let toolchain_env = toolchain_layer
        .env
        .apply(Scope::Build, &Env::from_current());

    let rust_toolchain_path = toolchain_env
        .get("PATH")
        .ok_or(libcnb::Error::BuildpackError(
            BuildpackError::RustToolchainPath,
        ))?;
    let cargo_metadata_info = MetadataCommand::new()
        .env("PATH", rust_toolchain_path)
        .manifest_path(context.app_dir.join("Cargo.toml"))
        .exec()
        .map_err(|err| libcnb::Error::BuildpackError(BuildpackError::CargoMetadata(err)))?;
    let targets = binary_targets(&cargo_metadata_info);

    let target_layer =
        context.handle_layer(layer_name!("target"), layers::Target { env: toolchain_env })?;

    prune_src(&context.app_dir)
        .map_err(|e| libcnb::Error::BuildpackError(BuildpackError::Io(e)))?;
    let mut launch = LaunchBuilder::new();
    for target in targets {
        fs::copy(
            target_layer.path.join("release").join(&target),
            context.app_dir.join(&target),
        )
        .map_err(|e| libcnb::Error::BuildpackError(BuildpackError::Io(e)))?;
        let process_type: ProcessType = target.parse().unwrap();
        launch.process(ProcessBuilder::new(process_type, [target]).build());
    }

    BuildResultBuilder::new().launch(launch.build()).build()
}

fn prune_src<P: AsRef<Path>>(app_dir: P) -> io::Result<()> {
    let app_dir = app_dir.as_ref();
    fs::remove_dir_all(app_dir.join("src"))?;
    fs::remove_file(app_dir.join("Cargo.toml"))?;
    fs::remove_file(app_dir.join("Cargo.lock"))?;

    Ok(())
}

fn binary_targets(metadata: &cargo_metadata::Metadata) -> Vec<String> {
    metadata
        // TODO: add support for workspaces
        .root_package()
        .map(|root_package| {
            root_package
                .targets
                .iter()
                .filter_map(|target| {
                    if target.is_bin() {
                        Some(target.name.clone())
                    } else {
                        None
                    }
                })
                .collect()
        })
        .unwrap_or_default()
}
