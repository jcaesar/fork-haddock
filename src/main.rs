use anyhow::{bail, Context, Result};
use clap::{crate_name, crate_version, Parser, Subcommand, ValueEnum};
use docker_compose_types::{Compose, ComposeFile, TopLevelVolumes};
use itertools::Itertools;
use serde_json::json;
use std::{collections::HashSet, fs};

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[command(subcommand)]
    command: Command,

    /// Compose configuration files
    #[arg(short, long)]
    file: Option<Vec<String>>,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// Converts the compose file to platform's canonical format
    #[command(alias = "config")]
    Convert {
        /// Format the output
        #[arg(long, value_enum, default_value_t = ConvertFormat::Yaml)]
        format: ConvertFormat,
        /// Only validate the configuration, don't print anything
        #[arg(short, long)]
        quiet: bool,
        /// Print the service names, one per line
        #[arg(long)]
        services: bool,
        /// Print the volume names, one per line
        #[arg(long)]
        volumes: bool,
        /// Print the profile names, one per line
        #[arg(long)]
        profiles: bool,
        /// Print the image names, one per line
        #[arg(long)]
        images: bool,
        /// Save to file (default to stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
    /// Print version information
    Version {
        /// Format the output
        #[arg(short, long, value_enum, default_value_t = VersionFormat::Pretty)]
        format: VersionFormat,
        /// Show only the version number
        #[arg(long)]
        short: bool,
    },
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
enum ConvertFormat {
    Yaml,
    Json,
}

#[derive(ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
enum VersionFormat {
    Pretty,
    Json,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let contents = match args.file {
        Some(paths) => paths
            .into_iter()
            .map(|path| {
                fs::read_to_string(&path)
                    .with_context(|| format!("{} not found", path))
                    .map(|content| (path, content))
            })
            .collect::<Result<Vec<_>, _>>()?,
        None => vec![(
            "compose.yaml".to_owned(),
            fs::read_to_string("compose.yaml")
                .or_else(|_| fs::read_to_string("compose.yml"))
                .or_else(|_| fs::read_to_string("docker-compose.yaml"))
                .or_else(|_| {
                    fs::read_to_string("docker-compose.yml").context("compose.yaml not found")
                })?,
        )],
    };
    let files = contents
        .into_iter()
        .map(|(path, content)| {
            serde_yaml::from_str::<ComposeFile>(&content)
                .with_context(|| format!("{} does not follow the Compose specification", path))
                .map(|file| (path, file))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut combined_file = Compose::new();

    for (path, file) in files {
        if let ComposeFile::V2Plus(file) = file {
            combined_file.version = file.version;
            combined_file.service = file.service;
            combined_file.extensions.extend(file.extensions);

            match (&mut combined_file.services, file.services) {
                (Some(combined_services), Some(services)) => combined_services.0.extend(services.0),
                (combined_services, services)
                    if combined_services.is_none() && services.is_some() =>
                {
                    *combined_services = services;
                }
                _ => {}
            }

            match (&mut combined_file.volumes, file.volumes) {
                (
                    Some(TopLevelVolumes::CV(combined_volumes)),
                    Some(TopLevelVolumes::CV(volumes)),
                ) => combined_volumes.0.extend(volumes.0),
                (
                    Some(TopLevelVolumes::Labelled(combined_volumes)),
                    Some(TopLevelVolumes::Labelled(volumes)),
                ) => combined_volumes.0.extend(volumes.0),
                (combined_volumes, volumes) if combined_volumes.is_none() && volumes.is_some() => {
                    *combined_volumes = volumes;
                }
                (_, None) => {}
                _ => bail!(
                    "{} uses a different volumes syntax from the other Compose files",
                    path
                ),
            }

            match (&mut combined_file.networks, file.networks) {
                (Some(combined_networks), Some(networks)) => combined_networks.0.extend(networks.0),
                (combined_networks, networks)
                    if combined_networks.is_none() && networks.is_some() =>
                {
                    *combined_networks = networks;
                }
                _ => {}
            }
        } else {
            bail!("{} does not follow the latest Compose specification", path);
        }
    }

    match args.command {
        Command::Convert {
            format,
            quiet,
            services,
            volumes,
            profiles,
            images,
            output,
        } => {
            if services {
                if let Some(services) = combined_file.services {
                    for service in services.0 {
                        println!("{}", service.0);
                    }
                }
            } else if volumes {
                match combined_file.volumes {
                    Some(TopLevelVolumes::CV(volumes)) => {
                        for volume in &volumes.0 {
                            println!("{}", volume.0);
                        }
                    }
                    Some(TopLevelVolumes::Labelled(volumes)) => {
                        for volume in &volumes.0 {
                            println!("{}", volume.0);
                        }
                    }
                    None => {}
                }
            } else if profiles {
                if let Some(services) = combined_file.services {
                    let mut all_profiles = HashSet::new();

                    for service in services.0 {
                        if let Some(profiles) = service.1.and_then(|service| service.profiles) {
                            all_profiles.extend(profiles);
                        }
                    }

                    for profile in all_profiles.into_iter().sorted() {
                        println!("{profile}");
                    }
                }
            } else if images {
                if let Some(services) = combined_file.services {
                    for service in services.0 {
                        if let Some(image) = service.1.and_then(|service| service.image) {
                            println!("{image}");
                        }
                    }
                }
            } else {
                match format {
                    ConvertFormat::Yaml => {
                        let contents = serde_yaml::to_string(&combined_file)?;

                        if !quiet && output.is_none() {
                            print!("{contents}");
                        }

                        if let Some(path) = output {
                            fs::write(path, contents)?;
                        }
                    }
                    ConvertFormat::Json => {
                        let contents = serde_json::to_string_pretty(&combined_file)?;

                        if !quiet && output.is_none() {
                            print!("{contents}");
                        }

                        if let Some(path) = output {
                            fs::write(path, contents)?;
                        }
                    }
                };
            }
        }
        Command::Version { format, short } => {
            if short {
                println!(crate_version!());
            } else {
                match format {
                    VersionFormat::Pretty => println!("{} {}", crate_name!(), crate_version!()),
                    VersionFormat::Json => println!("{}", json!({ "version": crate_version!() })),
                }
            }
        }
    }

    Ok(())
}
