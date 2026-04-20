//! Subcommands: status / stop / probe.

use std::process::ExitCode;

use anyhow::anyhow;

use crate::client::EmbedClient;
use crate::protocol::{Op, OpResult, Request};

pub async fn status() -> anyhow::Result<ExitCode> {
    let client = EmbedClient::new();
    let req = Request::new(Op::Ping);
    match client.query_async(req).await {
        Ok(resp) => match resp.result {
            OpResult::Pong {
                uptime_secs,
                request_count,
                rss_bytes,
                model_name,
            } => {
                println!("myth-embed daemon");
                println!("  Socket:    {:?}", myth_common::embed_socket_path());
                println!("  Model:     {}", model_name);
                println!("  Uptime:    {}s", uptime_secs);
                println!("  Requests:  {}", request_count);
                println!("  RSS:       {:.1} MB", rss_bytes as f64 / 1_048_576.0);
                Ok(ExitCode::SUCCESS)
            }
            OpResult::Error { code, message } => {
                eprintln!("daemon error {}: {}", code, message);
                Ok(ExitCode::from(1))
            }
            other => {
                eprintln!("unexpected result: {:?}", other);
                Ok(ExitCode::from(1))
            }
        },
        Err(_) => {
            println!("myth-embed daemon is not running");
            Ok(ExitCode::from(1))
        }
    }
}

pub async fn stop() -> anyhow::Result<ExitCode> {
    let client = EmbedClient::new();
    let req = Request::new(Op::Shutdown);
    match client.query_async(req).await {
        Ok(resp) => match resp.result {
            OpResult::ShuttingDown => {
                println!("myth-embed daemon stopping");
                Ok(ExitCode::SUCCESS)
            }
            other => {
                eprintln!("unexpected shutdown response: {:?}", other);
                Ok(ExitCode::from(1))
            }
        },
        Err(e) => Err(anyhow!("failed to reach daemon: {:#}", e)),
    }
}

pub async fn probe(args: &[String]) -> anyhow::Result<ExitCode> {
    let text = args.join(" ");
    if text.is_empty() {
        eprintln!("usage: myth-embed probe <text>");
        return Ok(ExitCode::from(2));
    }

    let client = EmbedClient::new();
    let vector = client.embed_async(&text).await?;

    let norm: f32 = vector.iter().map(|v| v * v).sum::<f32>().sqrt();
    println!("Text: {}", text);
    println!("Vector (first 5): {:?}", &vector[..5]);
    println!("Norm: {:.4}", norm);
    println!("Dim: {}", vector.len());
    Ok(ExitCode::SUCCESS)
}
