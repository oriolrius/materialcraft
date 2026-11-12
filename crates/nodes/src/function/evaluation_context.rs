use materialcraft_core::Value;
use materialcraft_image::Image;
use std::collections::BTreeMap;

/// Inputs shared by one function evaluation
pub struct EvaluationContext<'a> {
    pub scope: super::FunctionScope,
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub images: &'a [Option<Image>],
    pub tile: bool,
    pub variables: &'a BTreeMap<String, Value>,
}

impl EvaluationContext<'_> {
    pub(super) fn variable(&self, name: &str, local: &BTreeMap<String, Value>) -> Result<Option<Value>, String> {
        if let Some(variable) = super::SystemVariable::named(name) {
            // A local name must never make a system variable available outside its caller's scope
            return variable.resolve(self);
        }

        // A drop-down parameter reads as the integer of its chosen option
        Ok(local.get(name).or_else(|| self.variables.get(name)).map(|value| match value {
            Value::Enum(option) => Value::Int(i32::try_from(*option).unwrap_or(i32::MAX)),
            value => value.clone(),
        }))
    }
}
