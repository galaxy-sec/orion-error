use derive_getters::Getters;
use std::{env::var, fmt::Display};
use thiserror::Error;

#[derive(Debug, Clone, Getters)]
pub struct WithContext {
    target: Option<String>,
    context: ErrContext,
}

impl WithContext {
    pub fn want<S: Into<String>>(target: S) -> Self {
        Self {
            target: Some(target.into()),
            context: ErrContext::default(),
        }
    }
    pub fn with<S: Into<String>>(&mut self, msg: S) {
        self.context.items.push(msg.into());
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

#[derive(Default, Error, Debug, Clone, PartialEq)]
pub struct ErrContext {
    pub items: Vec<String>,
}

pub trait ContextAdd<T> {
    fn add_context(&mut self, msg: T);
}

impl Display for ErrContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.items.is_empty() {
            writeln!(f, "\nerror context:")?;
        }
        for i in &self.items {
            writeln!(f, "\t{}", i)?;
        }
        Ok(())
    }
}
