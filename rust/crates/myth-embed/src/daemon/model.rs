//! fastembed wrapper.
//!
//! NOTE: fastembed 5.13.2 uses the **builder** pattern for `InitOptions`
//! (a `TextInitOptions` type alias — fastembed splits text/image
//! backends). Do not use the struct-literal form; see
//! `docs/04-CRATES/06-myth-embed.md` change box "InitOptions builder".

use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Context};
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

pub struct Model {
    // fastembed's `TextEmbedding::embed(&mut self, ...)` mutates internal
    // state, so shared access needs a Mutex. Embed calls already run
    // inside `spawn_blocking`, and individual embeddings finish in ~10ms
    // — serialising them is fine for myth's call volume.
    inner: Arc<Mutex<TextEmbedding>>,
}

impl Model {
    pub async fn load() -> anyhow::Result<Arc<Self>> {
        let cache_dir = myth_common::myth_home().join("embeddings").join("models");
        std::fs::create_dir_all(&cache_dir).context("creating model cache dir")?;

        let model = tokio::task::spawn_blocking(move || {
            TextEmbedding::try_new(
                InitOptions::new(EmbeddingModel::MultilingualE5Small)
                    .with_cache_dir(cache_dir)
                    .with_show_download_progress(true),
            )
        })
        .await
        .context("join spawn_blocking")?
        .context("TextEmbedding::try_new failed")?;

        Ok(Arc::new(Self {
            inner: Arc::new(Mutex::new(model)),
        }))
    }

    pub async fn embed(&self, text: &str) -> anyhow::Result<[f32; 384]> {
        let text = text.to_string();
        let inner = self.inner.clone();
        tokio::task::spawn_blocking(move || -> anyhow::Result<[f32; 384]> {
            let mut guard = inner.lock().map_err(|_| anyhow!("model mutex poisoned"))?;
            let docs = vec![text];
            let embeddings = guard.embed(docs, None).context("embed call failed")?;
            let vec: Vec<f32> = embeddings
                .into_iter()
                .next()
                .ok_or_else(|| anyhow!("embed returned no vectors"))?;
            let len = vec.len();
            <[f32; 384]>::try_from(vec).map_err(|_| {
                anyhow!("expected 384-dim vector, got {}", len)
            })
        })
        .await
        .context("join spawn_blocking")?
    }
}
