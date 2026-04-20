//! usearch backed store — alternative Milestone B path if
//! sqlite-vec's DiskANN proves unstable (DECISIONS.md Decision 1).

use anyhow::bail;

pub struct UsearchStore;

impl UsearchStore {
    pub fn new() -> anyhow::Result<Self> {
        bail!("UsearchStore activates at Milestone B — see DECISIONS.md Decision 1")
    }
}
