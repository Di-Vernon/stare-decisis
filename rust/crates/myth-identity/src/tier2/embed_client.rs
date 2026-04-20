//! Thin adapter around `myth_embed::EmbedClient`. Exists so that
//! `IdentityMatcher` consumers can inject either a real daemon-backed
//! client or a test double without touching `myth-embed` internals.

use crate::store::Embedding;

pub struct EmbedAdapter {
    inner: myth_embed::EmbedClient,
}

impl EmbedAdapter {
    pub fn new() -> Self {
        Self {
            inner: myth_embed::EmbedClient::new(),
        }
    }

    pub fn with_client(client: myth_embed::EmbedClient) -> Self {
        Self { inner: client }
    }

    pub async fn embed(&self, text: &str) -> anyhow::Result<Embedding> {
        self.inner.embed_async(text).await
    }

    /// Synchronous helper for callers without a tokio context.
    pub fn embed_blocking(&self, text: &str) -> anyhow::Result<Embedding> {
        self.inner.embed(text)
    }
}

impl Default for EmbedAdapter {
    fn default() -> Self {
        Self::new()
    }
}
