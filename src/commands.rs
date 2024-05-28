use tauri::{command, AppHandle, Runtime, State, Window};

#[cfg(target_os = "windows")]
use crate::windows;

use crate::{shared::Program, MyState, Result};

#[command]
pub(crate) fn get_program_list<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    _state: State<'_, MyState<R>>,
) -> Result<Vec<Program>> {
    windows::get_program_list()
}

#[command]
pub(crate) fn get_program_by_path<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    _state: State<'_, MyState<R>>,
    path: String,
) -> Result<Program> {
    windows::get_program_by_path(path)
}

#[command]
pub(crate) fn suspend<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    state: State<'_, MyState<R>>,
) {
    state.watcher.suspend();
}

#[command]
pub(crate) fn resume<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    state: State<'_, MyState<R>>,
) {
    state.watcher.resume();
}
#[command]
pub(crate) fn is_active<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    state: State<'_, MyState<R>>,
    path: String,
) -> bool {
    state.watcher.is_active(path)
}
