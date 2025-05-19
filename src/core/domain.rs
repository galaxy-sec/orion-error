use std::fmt::Display;

use derive_more::From;
use serde::Serialize;
use thiserror::Error;

use super::UvsReason;

pub trait DomainReason: PartialEq + Display + Serialize {}

impl<T> DomainReason for T where T: From<UvsReason> + Display + PartialEq + Serialize {}

#[derive(Debug, PartialEq, Serialize, Error, From)]
pub enum NullReason {
    #[allow(dead_code)]
    #[error("null")]
    Null,
    #[error("{0}")]
    Uvs(UvsReason),
}
