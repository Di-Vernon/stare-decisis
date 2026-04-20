//! Surface rules — `~/.myth/surface-rules.yaml` (plus per-project
//! `.myth/surface-rules.yaml` merging, deferred to Wave 5 when the CLI
//! resolves project roots).

use crate::rules::compile::CompiledRules;

pub fn load() -> anyhow::Result<CompiledRules> {
    CompiledRules::load(&myth_common::surface_rules_path(), "surface")
}
