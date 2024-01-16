use tauri::{
  plugin::{Builder, TauriPlugin},
  Manager, Runtime,
};

use std::{collections::HashMap, sync::{Mutex, Arc}, thread};

#[macro_use]
extern crate log;

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;
mod event;
mod watcher;
mod timer;

use watcher::Watcher;

#[cfg(target_os = "windows")]
mod windows;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::ShionWatcher;
#[cfg(mobile)]
use mobile::ShionWatcher;

struct MyState<R: Runtime> {
  watcher: Arc<Watcher<R>>
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the shion-watcher APIs.
pub trait ShionWatcherExt<R: Runtime> {
  fn shion_watcher(&self) -> &ShionWatcher<R>;
}

impl<R: Runtime, T: Manager<R>> crate::ShionWatcherExt<R> for T {
  fn shion_watcher(&self) -> &ShionWatcher<R> {
    self.state::<ShionWatcher<R>>().inner()
  }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("shion-watcher")
    .invoke_handler(tauri::generate_handler![
      commands::get_program_list,
      commands::suspend,
      commands::resume,
      commands::is_active,
      ])
    .setup(|app, api| {
      #[cfg(mobile)]
      let shion_watcher = mobile::init(app, api)?;
      #[cfg(desktop)]
      let shion_watcher = desktop::init(app, api)?;
      app.manage(shion_watcher);

      let watcher = Watcher::new(app.clone());

      #[cfg(desktop)]
      thread::spawn({
        let watcher = watcher.clone();
        move || {
          watcher.run();
      }
      });

      // manage state so it is accessible by the commands
      app.manage(MyState {
        watcher
      });
      Ok(())
    })
    .build()
}
