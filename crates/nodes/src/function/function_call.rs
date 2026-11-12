use super::{FunctionDefinition, Program};

pub(crate) const DEFINITION: FunctionDefinition = FunctionDefinition { id: "instance", label: "Function instance", inputs: &[], category: "Graph" };

pub(crate) struct FunctionCall {
    pub program: Program,
    pub inputs: Vec<String>,
    pub defaults: std::collections::BTreeMap<String, materialcraft_core::Value>,
    /// The called graph sets variables, directly or in a graph it calls; the caller reads them after the call
    pub exports: bool,
}
