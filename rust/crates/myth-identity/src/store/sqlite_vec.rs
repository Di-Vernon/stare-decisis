//! sqlite-vec backed store — activated at Milestone B when in-memory
//! KNN P99 exceeds 50ms AND records > 20K (Decision 1).

use anyhow::bail;

pub struct SqliteVecStore;

impl SqliteVecStore {
    pub fn new() -> anyhow::Result<Self> {
        bail!("SqliteVecStore activates at Milestone B — see DECISIONS.md Decision 1")
    }
}
