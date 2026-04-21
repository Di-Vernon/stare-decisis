//! 자식 프로세스의 stdout/stderr 병렬 수집.

use anyhow::Result;
use std::time::Instant;
use tokio::io::{AsyncReadExt, BufReader};
use tokio::process::Child;

pub struct OutputCapture {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub last_activity: Instant,
}

impl OutputCapture {
    /// 자식의 stdout/stderr를 동시에 EOF까지 수집.
    ///
    /// 호출자가 `child.wait()` 로 종료 코드를 수령해야 한다.
    pub async fn capture_from_child(child: &mut Child) -> Result<OutputCapture> {
        let mut stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow::anyhow!("child stdout not piped"))?;
        let mut stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow::anyhow!("child stderr not piped"))?;

        let stdout_task = tokio::spawn(async move {
            let mut buf = Vec::new();
            let mut reader = BufReader::new(&mut stdout);
            reader.read_to_end(&mut buf).await?;
            Ok::<Vec<u8>, std::io::Error>(buf)
        });

        let stderr_task = tokio::spawn(async move {
            let mut buf = Vec::new();
            let mut reader = BufReader::new(&mut stderr);
            reader.read_to_end(&mut buf).await?;
            Ok::<Vec<u8>, std::io::Error>(buf)
        });

        let (stdout_res, stderr_res) = tokio::try_join!(stdout_task, stderr_task)?;

        Ok(OutputCapture {
            stdout: stdout_res?,
            stderr: stderr_res?,
            last_activity: Instant::now(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Stdio;
    use tokio::process::Command;

    #[tokio::test]
    async fn capture_stdout_stderr() {
        let mut child = Command::new("sh")
            .arg("-c")
            .arg("echo hello; echo err 1>&2")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("sh spawn");

        let cap = OutputCapture::capture_from_child(&mut child).await.unwrap();
        let _ = child.wait().await;

        assert_eq!(String::from_utf8_lossy(&cap.stdout).trim(), "hello");
        assert_eq!(String::from_utf8_lossy(&cap.stderr).trim(), "err");
    }
}
