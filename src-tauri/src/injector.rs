use anyhow::Result;

use crate::platform::macos;

pub fn copy_text_to_clipboard(text: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        macos::write_clipboard(text)?;
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = text;
        Ok(())
    }
}

pub fn deliver_text(text: &str, paste_mode: &str) -> Result<String> {
    copy_text_to_clipboard(text)?;

    if paste_mode == "auto-paste" {
        #[cfg(target_os = "macos")]
        {
            match macos::trigger_cmd_v_paste() {
                Ok(()) => return Ok("auto-paste".to_string()),
                Err(err) => {
                    eprintln!(
                        "auto-paste failed (falling back to clipboard): {err:#}. \
Grant Accessibility and Automation permissions if you want keystroke paste."
                    );
                }
            }
        }
    }

    Ok("clipboard".to_string())
}
