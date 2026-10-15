//! Agent hooks aligned with Rig's `force_tool_first_turn` example.
//!
//! Forcing `tool_choice = Required` on **every** turn loops until max_turns.
//! Gate on `ctx.turn() == 1` so turn 1 must call a tool, later turns can answer.

use rig::agent::{AgentHook, CompletionCallAction, CompletionCallEvent, HookContext, RequestPatch};
use rig::message::ToolChoice;

/// Nudge the model to call a DB tool on the first turn only (official Rig pattern).
#[derive(Clone, Debug, Default)]
pub struct ForceToolOnFirstTurn;

impl AgentHook for ForceToolOnFirstTurn {
    async fn on_completion_call(
        &self,
        ctx: &HookContext,
        _event: CompletionCallEvent<'_>,
    ) -> CompletionCallAction {
        if ctx.turn() == 1 {
            CompletionCallAction::patch(RequestPatch::new().tool_choice(ToolChoice::Required))
        } else {
            CompletionCallAction::continue_run()
        }
    }
}
