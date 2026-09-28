use std::time::Duration;

use serde_json::Value;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

use crate::config::Config;
use crate::error::{Error, Result};

pub async fn run(config: &Config, slug: &str) -> Result<String> {
    let message = format!("{slug} のタスクを get_task で取得し、その content に従って実行せよ。");
    let session_key = format!("agent:worker:simple-task:{slug}");
    let raw = run_command(
        &config.openclaw_bin,
        &[
            "agent",
            "--agent",
            "worker",
            "--session-key",
            session_key.as_str(),
            "--message",
            message.as_str(),
            "--json",
        ],
        config.timeout_secs,
    )
    .await?;
    extract_result(&raw)
}

async fn run_command(bin: &str, args: &[&str], timeout_secs: u64) -> Result<String> {
    let mut child = Command::new(bin)
        .args(args)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()?;
    let mut stdout = child
        .stdout
        .take()
        .ok_or_else(|| Error::Handoff("stdout を開けません".to_owned()))?;
    let mut stderr = child
        .stderr
        .take()
        .ok_or_else(|| Error::Handoff("stderr を開けません".to_owned()))?;
    let mut out = String::new();
    let mut err = String::new();
    let run = async {
        let (o, e) = tokio::join!(
            stdout.read_to_string(&mut out),
            stderr.read_to_string(&mut err),
        );
        o?;
        e?;
        child.wait().await
    };
    match tokio::time::timeout(Duration::from_secs(timeout_secs), run).await {
        Err(_) => Err(Error::Timeout(timeout_secs)),
        Ok(Err(e)) => Err(Error::Io(e)),
        Ok(Ok(status)) if !status.success() => Err(Error::Handoff(format!(
            "{bin} が失敗しました (exit={status}): {}",
            err.trim()
        ))),
        Ok(Ok(_)) => Ok(out),
    }
}

fn extract_result(raw: &str) -> Result<String> {
    let value: Value = serde_json::from_str(raw.trim())?;
    if value.get("ok").and_then(Value::as_bool) == Some(false) {
        return Err(Error::Handoff(
            value
                .pointer("/error/message")
                .and_then(Value::as_str)
                .unwrap_or("worker がエラーを返しました")
                .to_owned(),
        ));
    }
    ["/result", "/final", "/payloads/0/text"]
        .iter()
        .find_map(|p| value.pointer(p).and_then(Value::as_str))
        .map(str::to_owned)
        .ok_or_else(|| Error::Handoff("--json 封筒から result を取り出せません".to_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_result_field() -> Result<()> {
        assert_eq!(extract_result(r#"{"ok":true,"result":"done"}"#)?, "done");
        Ok(())
    }

    #[test]
    fn falls_back_to_final_then_payloads() -> Result<()> {
        assert_eq!(
            extract_result(r#"{"ok":true,"final":"final text"}"#)?,
            "final text"
        );
        assert_eq!(
            extract_result(r#"{"ok":true,"payloads":[{"text":"payload text"}]}"#)?,
            "payload text"
        );
        Ok(())
    }

    #[test]
    fn reports_failure_envelope() {
        assert!(matches!(
            extract_result(r#"{"ok":false,"error":{"message":"boom"}}"#),
            Err(Error::Handoff(m)) if m == "boom"
        ));
    }

    #[tokio::test]
    async fn runs_command_and_extracts_result() -> Result<()> {
        let raw = run_command("echo", &[r#"{"ok":true,"result":"from echo"}"#], 10).await?;
        assert_eq!(extract_result(&raw)?, "from echo");
        Ok(())
    }

    #[tokio::test]
    async fn timeout_kills_child() {
        assert!(matches!(
            run_command("sleep", &["30"], 1).await,
            Err(Error::Timeout(1))
        ));
    }
}
