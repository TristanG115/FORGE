// Undo/redo history module for the editor.

use crate::Canvas;
use serde::{Deserialize, Serialize};
use tracing::{debug, trace};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct History {
    pub states: Vec<Canvas>,
    pub current_index: usize,

    #[serde(default = "default_max_states")]
    pub max_states: usize,
}

fn default_max_states() -> usize {
    50
}

impl History {
    pub fn nnew(intial_state: Canvas) -> Self {}
}
