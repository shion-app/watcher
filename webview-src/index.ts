import { invoke } from '@tauri-apps/api/core'
import { EventCallback, listen } from '@tauri-apps/api/event'

interface Program {
  name: string
  path: string
  icon: Array<number>
}

interface WindowStatus {
  path: string,
  active: boolean,
  time: number
}

export function getProgramList() {
  return invoke<Array<Program>>('plugin:shion-watcher|get_program_list')
}

export function suspend() {
  return invoke('plugin:shion-watcher|suspend')
}

export function resume() {
  return invoke('plugin:shion-watcher|resume')
}

export function checkWatched() {
  return invoke('plugin:shion-watcher|check_watched')
}

export function onStatusChanged(fn: EventCallback<WindowStatus>) {
  return listen('plugin:shion-watcher://status-changed', fn)
}
