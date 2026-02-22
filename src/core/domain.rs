use std::fmt::Display;

use derive_more::From;
use thiserror::Error;

use super::UvsReason;

pub trait DomainReason: PartialEq + Display {}

impl<T> DomainReason for T where T: From<UvsReason> + Display + PartialEq {}

#[allow(dead_code)]
#[derive(Debug, PartialEq, Error, From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum NullReason {
    #[allow(dead_code)]
    #[error("null")]
    Null,
    #[error("{0}")]
    Uvs(UvsReason),
}
