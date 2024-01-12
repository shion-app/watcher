import { invoke } from '@tauri-apps/api/core'

export async function execute() {
  await invoke('plugin:shion-watcher|execute')
}
