use std::collections::HashMap;

use anyhow::Result;
use futures::{stream::FuturesUnordered, TryStreamExt};
use indexmap::IndexMap;
use itertools::Itertools;
use petgraph::{graphmap::DiGraphMap, Direction};
use tokio::sync::{broadcast, Barrier};

use crate::{
    compose::types::Compose,
    config::Config,
    podman::{types::Container, Podman},
    progress::{Finish, Progress},
};

/// Restart service containers
#[derive(clap::Args, Debug)]
#[command(next_display_order = None)]
pub(crate) struct Args {
    services: Vec<String>,

    /// Specify a shutdown timeout in seconds
    #[arg(short, long, default_value_t = 10)]
    timeout: u32,
}

async fn restart_containers(
    podman: &Podman,
    progress: &Progress,
    file: &Compose,
    containers: &HashMap<String, Vec<String>>,
    args: Args,
) -> Result<()> {
    let dependencies = &file
        .services
        .iter()
        .filter(|(service, _)| containers.keys().contains(service))
        .flat_map(|(to, service)| {
            service
                .depends_on
                .keys()
                .filter(|service| containers.keys().contains(service))
                .map(move |from| (from, to, ()))
        })
        .collect::<DiGraphMap<_, _>>();
    let capacity = dependencies
        .nodes()
        .map(|service| {
            dependencies
                .neighbors_directed(service, Direction::Incoming)
                .count()
        })
        .max()
        .unwrap_or_default()
        .max(1);
    let txs = &containers
        .keys()
        .map(|service| (service, broadcast::channel(capacity).0))
        .collect::<IndexMap<_, _>>();
    let barrier = &Barrier::new(containers.values().map(Vec::len).sum());

    containers
        .iter()
        .map(|(service, containers)| async move {
            containers
                .iter()
                .map(|container| async move {
                    let spinner =
                        progress.add_spinner(format!("Container {container}"), "Restarting");
                    let mut rx = txs[service].subscribe();

                    barrier.wait().await;

                    for _ in dependencies.neighbors_directed(service, Direction::Incoming) {
                        rx.recv().await?;
                    }

                    podman
                        .run(["restart", "--time", &args.timeout.to_string(), container])
                        .await
                        .finish_with_message(spinner, "Restarted")
                })
                .collect::<FuturesUnordered<_>>()
                .try_collect::<Vec<_>>()
                .await?;

            for dependent in dependencies.neighbors(service) {
                txs[dependent].send(())?;
            }

            Ok(())
        })
        .collect::<FuturesUnordered<_>>()
        .try_collect::<Vec<_>>()
        .await
        .map(|_| ())
}

pub(crate) async fn run(
    args: Args,
    podman: &Podman,
    file: &Compose,
    config: &Config,
) -> Result<()> {
    let output = podman
        .force_run([
            "ps",
            "--all",
            "--format",
            "json",
            "--filter",
            "label=io.podman.compose.oneoff=false",
            "--filter",
            &format!("pod={}", file.name.as_ref().unwrap()),
        ])
        .await?;
    let containers = serde_json::from_str::<Vec<Container>>(&output)?
        .into_iter()
        .filter_map(|mut container| {
            container
                .labels
                .and_then(|labels| labels.service)
                .and_then(|service| {
                    if args.services.contains(&service)
                        || (args.services.is_empty() && file.services.keys().contains(&service))
                    {
                        container.names.pop_front().map(|name| (service, name))
                    } else {
                        None
                    }
                })
        })
        .into_group_map();

    if !containers.is_empty() {
        let progress = Progress::new(config);

        restart_containers(podman, &progress, file, &containers, args).await?;

        progress.finish();
    }

    Ok(())
}
