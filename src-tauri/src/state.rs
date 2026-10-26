//! Shared application state.
use crate::settings::{Paths, Settings};
use nhse_core::HorizonSave;
use std::sync::Mutex;

pub struct Session {
    pub save: HorizonSave,
    /// Bumped on every mutation; the UI uses it to know when cached images are stale.
    pub revision: u64,
    pub dirty: bool,
}

pub struct AppState {
    pub session: Mutex<Option<Session>>,
    pub settings: Mutex<Settings>,
    pub paths: Paths,
}

impl AppState {
    pub fn new(paths: Paths) -> Self {
        let settings = Settings::load(&paths.settings_file());
        Self {
            session: Mutex::new(None),
            settings: Mutex::new(settings),
            paths,
        }
    }

    pub fn lang(&self) -> String {
        self.settings
            .lock()
            .map(|s| s.language.clone())
            .unwrap_or_else(|_| "en".into())
    }
}

/// Runs `f` with the open save (read only).
#[macro_export]
macro_rules! with_save {
    ($state:expr, |$s:ident| $body:expr) => {{
        let guard = $state
            .session
            .lock()
            .map_err(|_| $crate::error::CmdError("state poisoned".into()))?;
        let sess = guard
            .as_ref()
            .ok_or_else(|| $crate::error::CmdError("No save is open.".into()))?;
        let $s = &sess.save;
        $body
    }};
}

/// Runs `f` with the open save (mutable) and marks it dirty.
#[macro_export]
macro_rules! with_save_mut {
    ($state:expr, |$s:ident| $body:expr) => {{
        let mut guard = $state
            .session
            .lock()
            .map_err(|_| $crate::error::CmdError("state poisoned".into()))?;
        let sess = guard
            .as_mut()
            .ok_or_else(|| $crate::error::CmdError("No save is open.".into()))?;
        sess.dirty = true;
        sess.revision += 1;
        let $s = &mut sess.save;
        $body
    }};
}
