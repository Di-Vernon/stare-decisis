//! Foundation rules — `~/.myth/foundation-rules.yaml`.

use crate::rules::compile::CompiledRules;

pub fn load() -> anyhow::Result<CompiledRules> {
    CompiledRules::load(&myth_common::foundation_rules_path(), "foundation")
}
