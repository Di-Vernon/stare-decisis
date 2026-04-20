//! Length-prefixed bincode framing.
//!
//! Wire format (see PROTOCOL.md §3):
//!   +---------------+------------------------+
//!   | u32 LE length | bincode payload (N)    |
//!   +---------------+------------------------+
//!       4 bytes        1 ≤ N ≤ 1_000_000
//!
//! **Protocol v1 — wire format frozen per PROTOCOL.md.
//! Do not modify without a v2 migration plan.**

use anyhow::{anyhow, Context};
use serde::{de::DeserializeOwned, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub const PROTOCOL_VERSION: u8 = 1;
pub const MAX_PAYLOAD_BYTES: u32 = 1_000_000;

pub async fn write_message<W, T>(w: &mut W, msg: &T) -> anyhow::Result<()>
where
    W: AsyncWriteExt + Unpin,
    T: Serialize,
{
    let payload = bincode::serialize(msg).context("bincode serialize")?;
    let len: u32 = payload.len().try_into().map_err(|_| {
        anyhow!("payload too large to frame: {} bytes", payload.len())
    })?;
    if len > MAX_PAYLOAD_BYTES {
        return Err(anyhow!(
            "payload exceeds {} byte cap ({} bytes)",
            MAX_PAYLOAD_BYTES,
            len
        ));
    }
    w.write_u32_le(len).await.context("write length prefix")?;
    w.write_all(&payload).await.context("write payload")?;
    w.flush().await.context("flush")?;
    Ok(())
}

pub async fn read_message<R, T>(r: &mut R) -> anyhow::Result<T>
where
    R: AsyncReadExt + Unpin,
    T: DeserializeOwned,
{
    let len = r.read_u32_le().await.context("read length prefix")?;
    if len == 0 {
        return Err(anyhow!("zero-length payload rejected"));
    }
    if len > MAX_PAYLOAD_BYTES {
        return Err(anyhow!(
            "incoming payload exceeds {} byte cap ({} bytes)",
            MAX_PAYLOAD_BYTES,
            len
        ));
    }
    let mut buf = vec![0u8; len as usize];
    r.read_exact(&mut buf).await.context("read payload")?;
    bincode::deserialize(&buf).context("bincode deserialize")
}
