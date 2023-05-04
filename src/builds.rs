use crate::{layers, BuildpackError, RustBuildpack};
use cargo::util::toml::TomlManifest;
use libcnb::{
    build::{BuildContext, BuildResult, BuildResultBuilder},
    data::{
        launch::{LaunchBuilder, ProcessBuilder, ProcessType},
        layer_name,
    },
    layer_env::Scope,
    Env,
};
use libherokubuildpack::log::log_info;
use std::{fs, io, path::Path};

pub fn build(context: BuildContext<RustBuildpack>) -> libcnb::Result<BuildResult, BuildpackError> {
    let toolchain_layer =
        context.handle_layer(layer_name!("rust-toolchain"), layers::RustToolchain)?;
    let toolchain_env = toolchain_layer
        .env
        .apply(Scope::Build, &Env::from_current());

    let target_layer =
        context.handle_layer(layer_name!("target"), layers::Target { env: toolchain_env })?;

    log_info("Parsing Cargo.toml");
    let cargo_manifest_raw = fs::read_to_string(context.app_dir.join("Cargo.toml")).unwrap();
    let cargo_manifest: TomlManifest = toml::from_str(&cargo_manifest_raw).unwrap();

    prune_src(&context.app_dir)
        .map_err(|e| libcnb::Error::BuildpackError(BuildpackError::Io(e)))?;
    let targets = vec!["hello-world"];
    let mut launch = LaunchBuilder::new();
    for target in targets {
        fs::copy(
            target_layer.path.join("release").join(target),
            context.app_dir.join(target),
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
