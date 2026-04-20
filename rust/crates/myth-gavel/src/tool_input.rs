//! Input passed into `Gavel::judge`.

use myth_common::SessionId;

#[derive(Debug, Clone)]
pub struct ToolInput {
    pub tool_name: String,
    pub session_id: SessionId,
    /// A JSON serialisation of the hook's `tool_input` object. The
    /// Gavel runs regexes against this string so that every nested
    /// field is in scope (Bash command, file paths, URLs, …).
    pub serialized: String,
}
