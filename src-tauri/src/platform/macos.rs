use anyhow::{anyhow, bail, Context, Result};
use core_graphics::{
    event::{CGEvent, CGEventFlags, CGEventTapLocation},
    event_source::{CGEventSource, CGEventSourceStateID},
};
use std::ffi::c_void;
use std::process::{Command, Stdio};

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
    static kAXTrustedCheckOptionPrompt: *const c_void;

    fn CFDictionaryCreateMutable(
        allocator: *const c_void,
        capacity: isize,
        key_callbacks: *const c_void,
        value_callbacks: *const c_void,
    ) -> *mut c_void;
    fn CFDictionarySetValue(dict: *mut c_void, key: *const c_void, value: *const c_void);
    fn CFRelease(cf: *const c_void);
    static kCFBooleanTrue: *const c_void;
}

pub fn accessibility_granted() -> bool {
    // Safety: AXIsProcessTrusted is a pure macOS system call with no side effects.
    unsafe { AXIsProcessTrusted() }
}

pub fn reset_permissions(bundle_id: &str) -> Result<()> {
    let status = Command::new("tccutil")
        .arg("reset")
        .arg("All")
        .arg(bundle_id)
        .status()
        .context("failed to execute tccutil reset")?;
    if !status.success() {
        bail!("tccutil reset failed");
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
    let options = unsafe {
        CFDictionaryCreateMutable(std::ptr::null(), 1, std::ptr::null(), std::ptr::null())
    };
    if options.is_null() {
        bail!("failed to build Accessibility prompt options");
    }
    unsafe {
        CFDictionarySetValue(options, kAXTrustedCheckOptionPrompt, kCFBooleanTrue);
    }
    let trusted = unsafe { AXIsProcessTrustedWithOptions(options.cast_const()) };
    unsafe {
        CFRelease(options.cast_const());
    }
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
