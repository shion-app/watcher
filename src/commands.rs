use tauri::{AppHandle, command, Runtime, State, Window};

use crate::{MyState, Result, windows::{Program, self}};

#[command]
pub(crate) async fn get_program_list<R: Runtime>(
  _app: AppHandle<R>,
  _window: Window<R>,
  _state: State<'_, MyState<R>>,
) -> Result<Vec<Program>> {
  #[cfg(windows)]
  return windows::get_program_list();

  Ok(vec![])
}

#[command]
pub(crate) fn suspend<R: Runtime>(
  _app: AppHandle<R>,
  _window: Window<R>,
  state: State<'_, MyState<R>>,
)  {
  state.watcher.suspend();
}

#[command]
pub(crate) fn resume<R: Runtime>(
  _app: AppHandle<R>,
  _window: Window<R>,
  state: State<'_, MyState<R>>,
)  {
  state.watcher.resume();
}
#[command]
pub(crate) fn check_watched<R: Runtime>(
  _app: AppHandle<R>,
  _window: Window<R>,
  state: State<'_, MyState<R>>,
  path: String
)  -> bool {
  state.watcher.check_watched(path)
}
