mod types;

use std::{ffi::OsStr, path::PathBuf, process::Command};

use anyhow::{anyhow, bail, Context, Result};
use itertools::Itertools;
use once_cell::sync::Lazy;

use self::types::Version;
use crate::config::Config;

static PODMAN_MIN_SUPPORTED_VERSION: Lazy<semver::Version> =
    Lazy::new(|| semver::Version::new(4, 3, 0));

#[derive(Debug)]
pub(crate) struct Podman {
    project_directory: PathBuf,
    verbose: bool,
}

impl Podman {
    pub(crate) fn new(config: &Config) -> Result<Self> {
        let podman = Podman {
            project_directory: config.project_directory.clone(),
            verbose: config.verbose,
        };
        let output = podman.output(["version", "--format", "json"])?;
        let version = serde_json::from_str::<Version>(&output)
            .with_context(|| anyhow!("Podman version not recognised"))?
            .client
            .version;

        if version < *PODMAN_MIN_SUPPORTED_VERSION {
            bail!(
                "Only Podman {} and above is supported: version {version} found",
                *PODMAN_MIN_SUPPORTED_VERSION
            );
        }

        Ok(podman)
    }

    fn command<I, S>(&self, args: I) -> Command
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut command = Command::new("podman");
        command.current_dir(&self.project_directory).args(args);

        command
    }

    pub(crate) fn run<I, S>(&self, args: I) -> Result<()>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.output(args).map(|_| ())
    }

    pub(crate) fn output<I, S>(&self, args: I) -> Result<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut command = self.command(args);

        let output = command.output().with_context(|| {
            anyhow!(
                "`{} {}` cannot be executed",
                command.get_program().to_string_lossy(),
                command.get_args().map(OsStr::to_string_lossy).join(" ")
            )
        })?;

        if self.verbose {
            println!(
                "`{} {}`",
                command.get_program().to_string_lossy(),
                command.get_args().map(OsStr::to_string_lossy).join(" ")
            )
        }

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Err(
                anyhow!("{}", String::from_utf8_lossy(&output.stderr)).context(anyhow!(
                    "`{} {}` returned an error",
                    command.get_program().to_string_lossy(),
                    command.get_args().map(OsStr::to_string_lossy).join(" ")
                )),
            )
        }
    }
}
