use materialcraft_core::Value;

pub(crate) enum EvaluationStep {
    Enter(usize),
    Apply {
        instruction: usize,
        base: usize,
    },
    Branch(usize),
    /// Keep the value on top as the result of this instruction for the rest of the run
    Store(usize),
    Sequence(usize),
    LoopStart(usize),
    LoopCondition {
        instruction: usize,
        iteration: u32,
        last: Value,
    },
    LoopBody {
        instruction: usize,
        iteration: u32,
    },
}
