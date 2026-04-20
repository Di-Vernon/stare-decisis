//! Bedrock rules — `~/.myth/bedrock-rules.yaml`.

use crate::rules::compile::CompiledRules;

pub fn load() -> anyhow::Result<CompiledRules> {
    CompiledRules::load(&myth_common::bedrock_rules_path(), "bedrock")
}
