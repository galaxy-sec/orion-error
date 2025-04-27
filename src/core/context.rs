use derive_getters::Getters;
use serde::Serialize;
use std::fmt::Display;
use thiserror::Error;

#[derive(Debug, Clone, Getters, Default)]
pub struct WithContext {
    target: Option<String>,
    context: ErrContext,
}

impl WithContext {
    pub fn new() -> Self {
        Self {
            target: None,
            context: ErrContext::default(),
        }
    }
    pub fn want<S: Into<String>>(target: S) -> Self {
        Self {
            target: Some(target.into()),
            context: ErrContext::default(),
        }
    }
    pub fn with<S1: Into<String>, S2: Into<String>>(&mut self, key: S1, val: S2) {
        self.context.items.push((key.into(), val.into()));
    }
}

impl From<String> for WithContext {
    fn from(value: String) -> Self {
        Self {
            target: Some(value),
            context: ErrContext::default(),
        }
    }
}

impl From<&str> for WithContext {
    fn from(value: &str) -> Self {
        Self {
            target: Some(value.to_string()),
            context: ErrContext::default(),
        }
    }
}

impl From<&WithContext> for WithContext {
    fn from(value: &WithContext) -> Self {
        value.clone()
    }
}

#[derive(Default, Error, Debug, Clone, PartialEq, Serialize)]
pub struct ErrContext {
    pub items: Vec<(String, String)>,
}

pub trait ContextAdd<T> {
    fn add_context(&mut self, val: T);
}

impl Display for ErrContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.items.is_empty() {
            writeln!(f, "\nerror context:")?;
        }
        for (k, v) in &self.items {
            writeln!(f, "\t{} : {}", k, v)?;
        }
        Ok(())
    }
}
