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
