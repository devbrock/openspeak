use anyhow::{bail, Context, Result};
use std::process::{Command, Stdio};

pub fn write_clipboard(text: &str) -> Result<()> {
    let mut child = Command::new("pbcopy")
        .stdin(Stdio::piped())
        .spawn()
        .context("failed to execute pbcopy")?;

    {
        let Some(stdin) = child.stdin.as_mut() else {
            bail!("missing stdin handle for pbcopy");
        };
        use std::io::Write;
        stdin
            .write_all(text.as_bytes())
            .context("failed writing text to pbcopy")?;
    }

    let status = child.wait().context("failed waiting on pbcopy")?;
    if !status.success() {
        bail!("pbcopy exited with failure");
    }
    Ok(())
}

pub fn trigger_cmd_v_paste() -> Result<()> {
    let script = r#"tell application "System Events" to keystroke "v" using command down"#;
    let status = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .status()
        .context("failed to execute osascript for auto-paste")?;
    if !status.success() {
        bail!("osascript auto-paste failed");
    }
    Ok(())
}
