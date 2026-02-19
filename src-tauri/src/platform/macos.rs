use anyhow::{anyhow, bail, Context, Result};
use core_foundation::{
    base::TCFType,
    boolean::CFBoolean,
    dictionary::CFMutableDictionary,
    string::CFString,
};
use core_graphics::{
    event::{CGEvent, CGEventFlags, CGEventTapLocation},
    event_source::{CGEventSource, CGEventSourceStateID},
};
use std::process::{Command, Stdio};

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXIsProcessTrustedWithOptions(options: core_foundation::dictionary::CFDictionaryRef) -> bool;
}

pub fn accessibility_granted() -> bool {
    // Safety: AXIsProcessTrusted is a pure macOS system call with no side effects.
    unsafe { AXIsProcessTrusted() }
}

pub fn reset_permissions(bundle_id: &str) -> Result<()> {
    for service in ["Accessibility", "Microphone"] {
        let status = Command::new("tccutil")
            .arg("reset")
            .arg(service)
            .arg(bundle_id)
            .status()
            .with_context(|| format!("failed to execute tccutil reset for {service}"))?;
        if !status.success() {
            bail!("tccutil reset failed for {service}");
        }
    }
    Ok(())
}

pub fn open_permissions_settings() -> Result<()> {
    let panes = [
        "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility",
        "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone",
    ];
    for pane in panes {
        let status = Command::new("open")
            .arg(pane)
            .status()
            .with_context(|| format!("failed to open settings pane: {pane}"))?;
        if !status.success() {
            bail!("open failed for settings pane: {pane}");
        }
    }
    Ok(())
}

pub fn prompt_accessibility_permission() -> Result<bool> {
    // Ask macOS to show the Accessibility trust prompt for this exact app identity.
    let key = CFString::new("AXTrustedCheckOptionPrompt");
    let value = CFBoolean::true_value();
    let mut options = CFMutableDictionary::new();
    options.set(key.as_CFType(), value.as_CFType());
    let trusted = unsafe { AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef()) };
    Ok(trusted)
}

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
    // macOS virtual key code for 'v' on ANSI layout.
    let key_v = 9u16;
    let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
        .map_err(|_| anyhow!("failed to create macOS event source for auto-paste"))?;

    let key_down = CGEvent::new_keyboard_event(source.clone(), key_v, true)
        .map_err(|_| anyhow!("failed to create key-down event for auto-paste"))?;
    key_down.set_flags(CGEventFlags::CGEventFlagCommand);
    key_down.post(CGEventTapLocation::HID);

    let key_up = CGEvent::new_keyboard_event(source, key_v, false)
        .map_err(|_| anyhow!("failed to create key-up event for auto-paste"))?;
    key_up.set_flags(CGEventFlags::CGEventFlagCommand);
    key_up.post(CGEventTapLocation::HID);

    Ok(())
}
