use tauri::{AppHandle, command, Runtime, State, Window};

use crate::{MyState, Result, windows::{Program, self}};

#[command]
pub(crate) async fn get_program_list<R: Runtime>(
  _app: AppHandle<R>,
  _window: Window<R>,
  state: State<'_, MyState>,
) -> Result<Vec<Program>> {
  #[cfg(windows)]
  return windows::get_program_list();

  Ok(vec![])
}
