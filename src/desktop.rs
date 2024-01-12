use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
  app: &AppHandle<R>,
  _api: PluginApi<R, C>,
) -> crate::Result<ShionWatcher<R>> {
  Ok(ShionWatcher(app.clone()))
}

/// Access to the shion-watcher APIs.
pub struct ShionWatcher<R: Runtime>(AppHandle<R>);

impl<R: Runtime> ShionWatcher<R> {
  pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
    Ok(PingResponse {
      value: payload.value,
    })
  }
}
