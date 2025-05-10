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
            target: None,
            context: ErrContext::from(value.to_string()),
        }
    }
}

impl From<(String, String)> for WithContext {
    fn from(value: (String, String)) -> Self {
        Self {
            target: None,
            context: ErrContext::from(value),
        }
    }
}

impl From<(&str, &str)> for WithContext {
    fn from(value: (&str, &str)) -> Self {
        Self {
            target: None,
            context: ErrContext::from(value),
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
impl From<String> for ErrContext {
    fn from(value: String) -> Self {
        Self {
            items: vec![("msg".into(), value)],
        }
    }
}
impl From<(String, String)> for ErrContext {
    fn from(value: (String, String)) -> Self {
        Self {
            items: vec![(value.0, value.1)],
        }
    }
}

impl From<(&str, &str)> for ErrContext {
    fn from(value: (&str, &str)) -> Self {
        Self {
            items: vec![(value.0.to_string(), value.1.to_string())],
        }
    }
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
