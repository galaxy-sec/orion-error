use derive_getters::Getters;
use serde::Serialize;
use std::{
    fmt::Display,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Clone, Getters, Default, Serialize, PartialEq)]
pub struct WithContext {
    target: Option<String>,
    context: ErrContext,
}
impl From<ErrContext> for WithContext {
    fn from(value: ErrContext) -> Self {
        Self {
            target: None,
            context: value,
        }
    }
}

impl Display for WithContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(target) = &self.target {
            writeln!(f, "target: {} ", target)?;
        }
        for (i, (k, v)) in self.context().items.iter().enumerate() {
            writeln!(f, "{}. {}: {} ", i + 1, k, v)?;
        }
        Ok(())
    }
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
    pub fn with_path<S1: Into<String>, S2: Into<PathBuf>>(&mut self, key: S1, val: S2) {
        self.context
            .items
            .push((key.into(), format!("{}", val.into().display())));
    }

    pub fn with_want<S: Into<String>>(&mut self, target: S) {
        self.target = Some(target.into())
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

impl From<&PathBuf> for WithContext {
    fn from(value: &PathBuf) -> Self {
        Self {
            target: None,
            context: ErrContext::from(format!("{}", value.display())),
        }
    }
}
impl From<(&str, &PathBuf)> for WithContext {
    fn from(value: (&str, &PathBuf)) -> Self {
        Self {
            target: None,
            context: ErrContext::from((value.0, format!("{}", value.1.display()))),
        }
    }
}

impl From<&Path> for WithContext {
    fn from(value: &Path) -> Self {
        Self {
            target: None,
            context: ErrContext::from(format!("{}", value.display())),
        }
    }
}
impl From<(&str, &Path)> for WithContext {
    fn from(value: (&str, &Path)) -> Self {
        Self {
            target: None,
            context: ErrContext::from((value.0, format!("{}", value.1.display()))),
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

impl From<(&str, String)> for WithContext {
    fn from(value: (&str, String)) -> Self {
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
            items: vec![("key".into(), value)],
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

impl From<(&str, String)> for ErrContext {
    fn from(value: (&str, String)) -> Self {
        Self {
            items: vec![(value.0.to_string(), value.1)],
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
