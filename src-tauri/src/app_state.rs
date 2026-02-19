use parking_lot::Mutex;

use crate::{
    audio::RecordingSession,
    config::{default_config, load_or_init_config},
    types::{AppConfig, AppStatus, RecordingState},
};

pub struct AppState {
    inner: Mutex<AppStateInner>,
}

pub struct AppStateInner {
    pub status: AppStatus,
    pub config: AppConfig,
    pub active_session: Option<RecordingSession>,
}

impl AppState {
    pub fn new() -> Self {
        let config = load_or_init_config().unwrap_or_else(|_| default_config());
        #[cfg(target_os = "macos")]
        let accessibility_granted = crate::platform::macos::accessibility_granted();
        #[cfg(not(target_os = "macos"))]
        let accessibility_granted = false;
        Self {
            inner: Mutex::new(AppStateInner {
                status: AppStatus {
                    recording_state: RecordingState::Idle,
                    model_ready: false,
                    microphone_granted: cfg!(target_os = "macos"),
                    accessibility_granted,
                    last_error: None,
                },
                config,
                active_session: None,
            }),
        }
    }

    pub fn with_lock<T>(&self, f: impl FnOnce(&mut AppStateInner) -> T) -> T {
        let mut guard = self.inner.lock();
        f(&mut guard)
    }
}
