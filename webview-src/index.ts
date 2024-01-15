import { invoke } from '@tauri-apps/api/core'
import { EventCallback, listen } from '@tauri-apps/api/event'

interface Program {
  name: string
  path: string
  icon: Array<number>
}

export  function getProgramList() {
  return  invoke<Array<Program>>('plugin:shion-watcher|get_program_list')
}

export function onWindowActivate(fn: EventCallback<string>) {
  return listen('plugin:shion-watcher://window-activate', fn)
}
