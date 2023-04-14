use std::{env, error::Error, str::FromStr};

use anyhow::{anyhow, Result};
use console::style;
use serde::Serialize;
use serde_with::formats::Separator;
use sha2::{Digest as _, Sha256};

pub(crate) fn parse_container_path<T, U>(s: &str) -> Result<(Option<T>, U)>
where
    T: FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    if let Some(pos) = s.find(':') {
        Ok((Some(s[..pos].parse()?), s[pos + 1..].parse()?))
    } else {
        Ok((None, s.parse()?))
    }
}

pub(crate) fn parse_key_val<T, U>(s: &str) -> Result<(T, U)>
where
    T: FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s.find('=').ok_or_else(|| {
        anyhow!(
            "no '{}' found in '{}'",
            style("=").yellow(),
            style(s).yellow()
        )
    })?;

    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

pub(crate) fn parse_key_val_opt<T, U>(s: &str) -> Result<(T, Option<U>)>
where
    T: FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    if let Some(pos) = s.find('=') {
        Ok((s[..pos].parse()?, Some(s[pos + 1..].parse()?)))
    } else {
        Ok((s.parse()?, None))
    }
}

pub(crate) trait Digest {
    fn digest(&self) -> String;
}

impl<T> Digest for T
where
    T: Serialize,
{
    fn digest(&self) -> String {
        format!(
            "{:x}",
            Sha256::digest(serde_yaml::to_string(self).unwrap().as_bytes())
        )
    }
}

pub(crate) struct PathSeparator;

impl Separator for PathSeparator {
    fn separator() -> &'static str {
        Box::leak(
            env::var("COMPOSE_PATH_SEPARATOR")
                .unwrap_or_else(|_| {
                    String::from(if cfg!(unix) {
                        ":"
                    } else if cfg!(windows) {
                        ";"
                    } else {
                        unreachable!()
                    })
                })
                .into_boxed_str(),
        )
    }
}
