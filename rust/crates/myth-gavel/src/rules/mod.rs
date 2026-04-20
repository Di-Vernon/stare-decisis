//! Rule loading + compilation. Three categories (Bedrock/Foundation/
//! Surface) are thin YAML-path wrappers around the shared `CompiledRules`
//! type in `compile.rs`.

pub mod bedrock;
pub mod compile;
pub mod foundation;
pub mod surface;

pub use compile::{CompiledRule, CompiledRules, Likelihood, RuleMatch};

pub struct RuleSet {
    pub bedrock: CompiledRules,
    pub foundation: CompiledRules,
    pub surface: CompiledRules,
}

impl RuleSet {
    pub fn load_all() -> anyhow::Result<Self> {
        Ok(Self {
            bedrock: bedrock::load()?,
            foundation: foundation::load()?,
            surface: surface::load()?,
        })
    }

    /// Construct directly from in-memory rule sets. Primary use is
    /// tests; `Gavel::init` uses `load_all`.
    pub fn from_parts(
        bedrock: CompiledRules,
        foundation: CompiledRules,
        surface: CompiledRules,
    ) -> Self {
        Self {
            bedrock,
            foundation,
            surface,
        }
    }
}
