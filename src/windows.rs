use std::{ffi::OsStr, process::Command, sync::Arc, thread, time::Duration};

use anyhow::bail;
use nodio_win32::{AudioSessionEvent, SessionState, Win32Context};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use windows::{
    core::PWSTR,
    Win32::{
        Foundation::{GetLastError, HWND, MAX_PATH, POINT},
        System::Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_INFORMATION,
            PROCESS_VM_READ,
        },
        UI::{
            Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
            WindowsAndMessaging::{
                DispatchMessageW, GetCursorPos, GetForegroundWindow, GetMessageW,
                GetWindowThreadProcessId, TranslateMessage, WindowFromPoint,
                EVENT_SYSTEM_FOREGROUND, MSG, WINEVENT_OUTOFCONTEXT,
            },
        },
    },
};

use crate::{
    watcher::{WatcherEvent, WATCHER_EVENT_CHANNEL, WATCHER_STATUS_CHANNEL},
    Result,
};

#[derive(Deserialize, Serialize, Debug)]
pub struct Program {
    path: String,
    name: String,
    icon: Vec<u8>,
}

fn powershell<S: AsRef<OsStr>>(script: S) -> Result<String> {
    let mut command = Command::new("powershell");
    command.arg("-c");
    command.arg(script);
    let output = command.output()?.stdout;
    let s = String::from_utf8_lossy(&output).to_string();
    Ok(s)
}

pub fn get_program_list() -> Result<Vec<Program>> {
    let s = powershell(include_str!("../powershell/get-program.ps1"))?;
    let list = serde_json::from_str(&s)?;
    Ok(list)
}

fn get_window_pid(hwnd: HWND) -> u32 {
    let mut pid: u32 = 0;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid as *mut u32)) };
    pid
}

pub fn get_program_path(hwnd: HWND) -> Option<String> {
    let pid = get_window_pid(hwnd);
    let handle =
        match unsafe { OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, None, pid) } {
            Ok(v) => v,
            Err(_) => return None,
        };
    let mut len: u32 = MAX_PATH;
    let mut name = vec![0u16; len as usize];
    let ret = unsafe {
        QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            PWSTR(name.as_mut_ptr()),
            &mut len,
        )
    };
    if ret.is_err() || len == 0 {
        return None;
    }
    unsafe { name.set_len(len as usize) };
    let module_path = String::from_utf16_lossy(&name);
    if module_path.is_empty() {
        return None;
    }
    Some(module_path)
}

pub fn get_mouse_area_program_path() -> Option<String> {
    let mut point = POINT { x: 0, y: 0 };
    let hwnd = unsafe {
        let _ = GetCursorPos(&mut point);
        WindowFromPoint(point)
    };
    get_program_path(hwnd)
}

pub fn get_foreground_program_path() -> Option<String> {
    let hwnd = unsafe { GetForegroundWindow() };
    get_program_path(hwnd)
}

#[derive(Debug)]
struct Watcher {
    hook: HWINEVENTHOOK,
}

impl Watcher {
    pub fn init() -> anyhow::Result<Self> {
        let hook = unsafe {
            SetWinEventHook(
                EVENT_SYSTEM_FOREGROUND,
                EVENT_SYSTEM_FOREGROUND,
                None,
                Some(win_event_proc),
                0,
                0,
                WINEVENT_OUTOFCONTEXT,
            )
        };
        if hook.is_invalid() {
            bail!("watcher SetWinEventHook error");
        }
        info!("watcher start");
        Ok(Self { hook })
    }
}

impl Drop for Watcher {
    fn drop(&mut self) {
        debug!("watcher drop");
        if !self.hook.is_invalid() {
            unsafe { UnhookWinEvent(self.hook) };
        }
    }
}

unsafe extern "system" fn win_event_proc(
    _h_win_event_hook: HWINEVENTHOOK,
    _event: u32,
    hwnd: HWND,
    id_object: i32,
    _id_child: i32,
    _dw_event_thread: u32,
    _dwms_event_time: u32,
) {
    if id_object != 0 {
        return;
    }

    let path = get_program_path(hwnd);
    if path.is_none() {
        return;
    }
    let path = path.unwrap();

    let _ = WATCHER_EVENT_CHANNEL.lock().0.send(WatcherEvent {
        path,
        is_audio: false,
        active: true,
    });
}

pub struct App;

impl App {
    fn eventloop() -> anyhow::Result<()> {
        let mut message = MSG::default();
        loop {
            let ret = unsafe { GetMessageW(&mut message, HWND(0), 0, 0) };
            match ret.0 {
                -1 => {
                    unsafe { GetLastError() }?;
                }
                0 => break,
                _ => unsafe {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                },
            }
        }

        Ok(())
    }

    pub fn start() -> anyhow::Result<()> {
        let _watcher = Watcher::init()?;
        watch_audio();
        Self::eventloop()
    }
}

fn watch_audio() {
    thread::spawn(|| {
        let context = Win32Context::new(|event, path| match event {
            AudioSessionEvent::StateChange(state) => {
                let active = state == SessionState::Active;
                let _ = WATCHER_EVENT_CHANNEL.lock().0.send(WatcherEvent {
                    path,
                    is_audio: true,
                    active,
                });
            }
            _ => {}
        });
        loop {
            if let Ok(event) = WATCHER_STATUS_CHANNEL.lock().1.try_recv() {
                if event.running {
                    let list = context.read().get_active_session_filename();
                    for path in list {
                        let _ = WATCHER_EVENT_CHANNEL.lock().0.send(WatcherEvent {
                            path,
                            is_audio: true,
                            active: true,
                        });
                    }
                }
            }
            thread::sleep(Duration::from_millis(1000 / 60));
        }
    });
}

mod tests {
    use super::*;

    #[test]
    fn test_get_program_list() {
        let list = get_program_list();
        println!("{:?}", list);
    }

    #[test]
    fn test_app() {
        if let Err(err) = App::start() {
            println!("watcher error: {}", err);
        }
    }
}
