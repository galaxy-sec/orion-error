use derive_getters::Getters;
use std::fmt::Display;
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
        self.context.items.push(msg.into())
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
