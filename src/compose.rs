use anyhow::{anyhow, Context, Result};
use byte_unit::Byte;
use humantime::{format_duration, parse_duration};
use indexmap::{IndexMap, IndexSet};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_with::{
    formats::SpaceSeparator, serde_as, serde_conv, skip_serializing_none, DisplayFromStr,
    DurationMicroSeconds, OneOrMany, PickFirst, StringWithSeparator,
};
use std::{convert::Infallible, fs, time::Duration};

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Default, Debug)]
pub(crate) struct Compose {
    pub(crate) version: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) services: IndexMap<String, Service>,
    pub(crate) volumes: Option<IndexMap<String, Option<Volume>>>,
}

impl Compose {
    pub(crate) fn new() -> Self {
        Default::default()
    }
}

#[serde_as]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub(crate) struct Service {
    pub(crate) blkio_config: Option<BlkioConfig>,
    pub(crate) cap_add: Option<Vec<String>>,
    pub(crate) cap_drop: Option<Vec<String>>,
    pub(crate) cgroup_parent: Option<String>,
    #[serde_as(as = "Option<PickFirst<(_, StringWithSeparator::<SpaceSeparator, String>)>>")]
    pub(crate) command: Option<Vec<String>>,
    pub(crate) container_name: Option<String>,
    pub(crate) cpuset: Option<String>,
    #[serde_as(as = "Option<PickFirst<(DurationMicroSeconds, DurationWithPrefix)>>")]
    pub(crate) cpu_period: Option<Duration>,
    #[serde_as(as = "Option<PickFirst<(DurationMicroSeconds, DurationWithPrefix)>>")]
    pub(crate) cpu_quota: Option<Duration>,
    #[serde_as(as = "Option<PickFirst<(DurationMicroSeconds, DurationWithPrefix)>>")]
    pub(crate) cpu_rt_period: Option<Duration>,
    #[serde_as(as = "Option<PickFirst<(DurationMicroSeconds, DurationWithPrefix)>>")]
    pub(crate) cpu_rt_runtime: Option<Duration>,
    pub(crate) cpu_shares: Option<i64>,
    #[serde_as(as = "Option<PickFirst<(_, DependsOnVec)>>")]
    pub(crate) depends_on: Option<IndexMap<String, DependsOn>>,
    pub(crate) device_cgroup_rules: Option<String>,
    pub(crate) devices: Option<String>,
    #[serde_as(as = "Option<OneOrMany<_>>")]
    pub(crate) dns: Option<Vec<String>>,
    pub(crate) dns_opt: Option<Vec<String>>,
    #[serde_as(as = "Option<OneOrMany<_>>")]
    pub(crate) dns_search: Option<Vec<String>>,
    #[serde_as(as = "Option<PickFirst<(_, StringWithSeparator::<SpaceSeparator, String>)>>")]
    pub(crate) entrypoint: Option<Vec<String>>,
    #[serde_as(as = "Option<OneOrMany<_>>")]
    pub(crate) env_file: Option<Vec<String>>,
    #[serde_as(as = "Option<PickFirst<(_, EnvironmentVec)>>")]
    pub(crate) environment: Option<IndexMap<String, Option<String>>>,
    pub(crate) expose: Option<Vec<String>>,
    pub(crate) extends: Option<IndexMap<String, Extends>>,
    pub(crate) external_links: Option<Vec<String>>,
    pub(crate) extra_hosts: Option<Vec<String>>,
    pub(crate) group_add: Option<Vec<String>>,
    pub(crate) healthcheck: Option<Healthcheck>,
    pub(crate) hostname: Option<String>,
    pub(crate) image: String,
    pub(crate) init: Option<bool>,
    pub(crate) ipc: Option<String>,
    #[serde_as(as = "Option<PickFirst<(_, LabelsVec)>>")]
    pub(crate) labels: Option<IndexMap<String, String>>,
    pub(crate) links: Option<String>,
    pub(crate) logging: Option<Logging>,
    pub(crate) mac_address: Option<String>,
    #[serde_as(as = "Option<PickFirst<(_, DisplayFromStr)>>")]
    pub(crate) memswap_limit: Option<Byte>,
    pub(crate) mem_swappiness: Option<i64>,
    pub(crate) network_mode: Option<String>,
    pub(crate) oom_kill_disable: Option<bool>,
    pub(crate) oom_score_adj: Option<i64>,
    pub(crate) pid: Option<String>,
    pub(crate) platform: Option<String>,
    #[serde_as(as = "Option<Vec<PickFirst<(_, PortOrString, PortOrU32)>>>")]
    pub(crate) ports: Option<Vec<Port>>,
    pub(crate) privileged: Option<bool>,
    pub(crate) profiles: Option<Vec<String>>,
    pub(crate) pull_policy: Option<String>,
    pub(crate) read_only: Option<bool>,
    pub(crate) restart: Option<String>,
    pub(crate) security_opt: Option<Vec<String>>,
    #[serde_as(as = "Option<PickFirst<(_, DisplayFromStr)>>")]
    pub(crate) shm_size: Option<Byte>,
    #[serde_as(as = "Option<DurationWithPrefix>")]
    pub(crate) stop_grace_period: Option<Duration>,
    pub(crate) stop_signal: Option<String>,
    #[serde_as(as = "Option<PickFirst<(_, SysctlsVec)>>")]
    pub(crate) sysctls: Option<IndexMap<String, String>>,
    #[serde_as(as = "Option<OneOrMany<_>>")]
    pub(crate) tmpfs: Option<Vec<String>>,
    pub(crate) tty: Option<bool>,
    pub(crate) ulimits: Option<IndexMap<String, Ulimits>>,
    pub(crate) user: Option<String>,
    pub(crate) userns_mode: Option<String>,
    pub(crate) volumes_from: Option<Vec<String>>,
    pub(crate) working_dir: Option<String>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub(crate) struct BlkioConfig {
    pub(crate) weight: Option<u16>,
    pub(crate) weight_device: Option<Vec<WeightDevice>>,
    pub(crate) device_read_bps: Option<Vec<ThrottleDevice>>,
    pub(crate) device_write_bps: Option<Vec<ThrottleDevice>>,
    pub(crate) device_read_iops: Option<Vec<ThrottleDevice>>,
    pub(crate) device_write_iops: Option<Vec<ThrottleDevice>>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub(crate) struct WeightDevice {
    pub(crate) path: String,
    pub(crate) weight: u16,
}

#[serde_as]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub(crate) struct ThrottleDevice {
    pub(crate) path: String,
    #[serde_as(as = "PickFirst<(_, DisplayFromStr)>")]
    pub(crate) rate: Byte,
}

serde_conv!(
    DurationWithPrefix,
    Duration,
    |duration: &Duration| format_duration(*duration).to_string(),
    |duration: String| parse_duration(&duration)
);

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct DependsOn {
    pub(crate) condition: Condition,
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Condition {
    #[serde(rename = "service_started")]
    Started,
    #[serde(rename = "service_healthy")]
    Healthy,
    #[serde(rename = "service_completed_successfully")]
    CompletedSuccessfully,
}

serde_conv!(
    DependsOnVec,
    IndexMap<String, DependsOn>,
    |dependencies: &IndexMap<String, DependsOn>| dependencies.keys().cloned().collect::<Vec<_>>(),
    |dependencies: Vec<String>| -> std::result::Result<_, Infallible> {
        Ok(IndexMap::from_iter(dependencies.into_iter().map(
            |dependency| {
                (
                    dependency,
                    DependsOn {
                        condition: Condition::Started,
                    },
                )
            },
        )))
    }
);

serde_conv!(
    EnvironmentVec,
    IndexMap<String, Option<String>>,
    |variables: &IndexMap<String, Option<String>>| {
        variables
            .iter()
            .map(|(key, value)| match value {
                Some(value) => format!("{key}={value}"),
                None => key.to_owned(),
            })
            .collect::<Vec<_>>()
    },
    |variables: Vec<String>| -> std::result::Result<_, Infallible> {
        Ok(IndexMap::from_iter(variables.into_iter().map(|variable| {
            let mut parts = variable.split('=');
            (
                parts.next().unwrap().to_owned(),
                parts.next().map(|part| part.to_owned()),
            )
        })))
    }
);

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub(crate) struct Extends {
    pub(crate) service: String,
    pub(crate) file: Option<String>,
}

#[serde_as]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub(crate) struct Healthcheck {
    #[serde_as(as = "PickFirst<(_, HealthcheckOneOrMany)>")]
    pub(crate) test: Vec<String>,
    #[serde_as(as = "Option<DurationWithPrefix>")]
    pub(crate) interval: Option<Duration>,
    #[serde_as(as = "Option<DurationWithPrefix>")]
    pub(crate) timeout: Option<Duration>,
    #[serde_as(as = "Option<DurationWithPrefix>")]
    pub(crate) start_period: Option<Duration>,
    pub(crate) retries: Option<u64>,
    pub(crate) disable: Option<bool>,
}

serde_conv!(
    HealthcheckOneOrMany,
    Vec<String>,
    |test: &Vec<String>| test.iter().skip(1).join(" "),
    |test: String| -> std::result::Result<_, Infallible> { Ok(vec!["CMD-SHELL".to_owned(), test]) }
);

serde_conv!(
    LabelsVec,
    IndexMap<String, String>,
    |variables: &IndexMap<String, String>| {
        variables
            .iter()
            .map(|(key, value)| {
                if value.is_empty() {
                    key.to_owned()
                } else {
                    format!("{key}={value}")
                }
            })
            .collect::<Vec<_>>()
    },
    |variables: Vec<String>| -> std::result::Result<_, Infallible> {
        Ok(IndexMap::from_iter(variables.into_iter().map(|variable| {
            let mut parts = variable.split('=');
            (
                parts.next().unwrap().to_owned(),
                parts.next().map(|part| part.to_owned()).unwrap_or_default(),
            )
        })))
    }
);

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub(crate) struct Logging {
    pub(crate) driver: Option<String>,
    pub(crate) options: Option<IndexMap<String, String>>,
}

#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub(crate) struct Port {
    pub(crate) target: u32,
    pub(crate) published: Option<String>,
    pub(crate) host_ip: Option<String>,
    pub(crate) protocol: Option<String>,
}

serde_conv!(
    PortOrString,
    Port,
    |port: &Port| {
        let mut string = port.target.to_string();

        if let Some(published) = port.published.to_owned() {
            string = format!("{published}:{string}");
        }

        if let Some(host_ip) = port.host_ip.to_owned() {
            string = format!("{host_ip}:{string}");
        }

        if let Some(protocol) = port.protocol.to_owned() {
            string = format!("{string}/{protocol}");
        }

        string
    },
    |port: String| -> Result<_> {
        let mut parts = port.split(':').rev();
        let container_port = parts.next().unwrap();
        let mut container_parts = container_port.split('/');
        let target = container_parts.next().unwrap();

        Ok(Port {
            target: target.parse()?,
            published: parts.next().and_then(|part| {
                if part.is_empty() {
                    None
                } else {
                    Some(part.to_owned())
                }
            }),
            host_ip: parts.next().and_then(|part| {
                if part.is_empty() {
                    None
                } else {
                    Some(part.to_owned())
                }
            }),
            protocol: container_parts.next().and_then(|part| {
                if part.is_empty() {
                    None
                } else {
                    Some(part.to_owned())
                }
            }),
        })
    }
);

serde_conv!(
    PortOrU32,
    Port,
    |_: &Port| 0,
    |target: u32| -> std::result::Result<_, Infallible> {
        Ok(Port {
            target,
            published: None,
            host_ip: None,
            protocol: None,
        })
    }
);

serde_conv!(
    SysctlsVec,
    IndexMap<String, String>,
    |variables: &IndexMap<String, String>| {
        variables
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
    },
    |variables: Vec<String>| -> Result<_> {
        let variables = variables.into_iter().map(|variable| -> Result<_> {
            let mut parts = variable.split('=');
            let key = parts.next().unwrap().to_owned();
            let value = parts.next().map(|part| part.to_owned()).ok_or_else(|| anyhow!("parameter not defined"))?;

            Ok((key, value))
        }).collect::<Result<Vec<_>, _>>()?;

        Ok(IndexMap::from_iter(variables.into_iter()))
    }
);

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
#[serde(untagged)]
pub(crate) enum Ulimits {
    Single(isize),
    Double { soft: isize, hard: isize },
}

#[serde_as]
#[skip_serializing_none]
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
pub(crate) struct Volume {
    pub(crate) driver: Option<String>,
    pub(crate) driver_opts: Option<IndexMap<String, String>>,
    pub(crate) external: Option<bool>,
    #[serde_as(as = "Option<PickFirst<(_, LabelsVec)>>")]
    pub(crate) labels: Option<IndexMap<String, String>>,
    pub(crate) name: Option<String>,
}

pub(crate) fn parse(paths: Option<Vec<String>>) -> Result<Compose> {
    let contents = match paths {
        Some(paths) => paths
            .into_iter()
            .map(|path| {
                fs::read_to_string(&path)
                    .with_context(|| format!("{path} not found"))
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
            let mut unused = IndexSet::new();

            serde_ignored::deserialize(serde_yaml::Deserializer::from_str(&content), |path| {
                unused.insert(path.to_string());
            })
            .with_context(|| format!("{path} does not follow the Compose specification"))
            .map(|file: Compose| (path, file, unused))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut combined_file = Compose::new();

    for (path, file, unused) in files {
        if !unused.is_empty() {
            eprintln!(
                "Warning: Unsupported/unknown attributes in {path}: {}",
                unused.into_iter().join(", ")
            );
        }

        combined_file.version = file.version;
        combined_file.name = file.name;
        combined_file.services.extend(file.services);

        match (&mut combined_file.volumes, file.volumes) {
            (Some(combined_volumes), Some(volumes)) => combined_volumes.extend(volumes),
            (combined_volumes, volumes) if combined_volumes.is_none() && volumes.is_some() => {
                *combined_volumes = volumes;
            }
            _ => {}
        }
    }

    Ok(combined_file)
}
