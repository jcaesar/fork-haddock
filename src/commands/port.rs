use std::fmt::{self, Display, Formatter};

use anyhow::{anyhow, bail, Result};
use clap::ValueEnum;

use crate::{
    compose::types::Compose,
    podman::{types::Container, Podman},
};

/// Print the public port for a port binding
#[derive(clap::Args, Debug)]
#[command(next_display_order = None)]
pub(crate) struct Args {
    service: String,
    port: u16,

    #[arg(long, value_enum, default_value_t = Protocol::Tcp)]
    protocol: Protocol,

    /// Index of the container if service has multiple replicas
    #[arg(long, default_value_t = 1)]
    index: usize,
}

#[derive(ValueEnum, Clone, Debug)]
enum Protocol {
    Tcp,
    Udp,
}

impl Display for Protocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format!("{self:?}").to_ascii_lowercase())
    }
}

pub(crate) async fn run(args: Args, podman: &Podman, file: &Compose) -> Result<()> {
    let name = file.name.as_ref().unwrap();

    let output = podman
        .force_run([
            "ps",
            "--all",
            "--format",
            "json",
            "--filter",
            &format!("pod={name}"),
            "--filter",
            &format!("label=io.podman.compose.service={}", args.service),
        ])
        .await?;
    let containers = serde_json::from_str::<Vec<Container>>(&output)?;

    if containers.is_empty() {
        bail!("No container found for service \"{}\"", args.service);
    }

    let container = containers
        .into_iter()
        .find_map(|mut container| {
            container
                .labels
                .and_then(|labels| labels.container_number)
                .and_then(|n| {
                    if n == args.index {
                        container.names.pop_front()
                    } else {
                        None
                    }
                })
        })
        .ok_or_else(|| {
            anyhow!(
                "Service \"{}\" is not running container #{}",
                args.service,
                args.index
            )
        })?;

    print!(
        "{}",
        podman
            .run([
                "port",
                &container,
                &format!("{}/{}", args.port, args.protocol)
            ])
            .await?
    );

    Ok(())
}
