use std::{
    collections::HashSet, os::windows::ffi::OsStrExt, path::Path, thread, time::Duration
};

use anyhow::{anyhow, bail};
use nodio_win32::{AudioSessionEvent, SessionState, Win32Context};
use windows::{
    core::{w, PCWSTR, PWSTR},
    Win32::{
        Foundation::{GetLastError, BOOL, HWND, LPARAM, MAX_PATH, POINT}, Storage::FileSystem::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW}, System::Threading::{
            OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
            PROCESS_QUERY_INFORMATION, PROCESS_VM_READ,
        }, UI::{
            Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
            WindowsAndMessaging::{
                DispatchMessageW, EnumWindows, GetCursorPos, GetForegroundWindow, GetMessageW, GetWindow, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible, TranslateMessage, WindowFromPoint, EVENT_SYSTEM_FOREGROUND, GWL_EXSTYLE, GW_OWNER, MSG, WINEVENT_OUTOFCONTEXT, WS_EX_TOPMOST
            },
        }
    },
};

use crate::{
    shared::Program,
    watcher::{WatcherEvent, WATCHER_EVENT_CHANNEL, WATCHER_STATUS_CHANNEL},
    Result
};

mod icons;

pub fn get_program_list() -> Result<Vec<Program>> {
    let processes = get_foreground_processes();
    let mut programs = Vec::new();

    for pid in processes {
        if let Some(path) = get_program_path(pid) {
            let file_path = Path::new(&path);
            let stem = file_path.file_stem().unwrap().to_str().unwrap().to_string();
            let name =  match get_display_name(file_path)  {
                Ok(name) => if name.is_empty() {
                    stem
                } else {
                    name
                },
                Err(_) => stem
            };
            let mut icon_cache = icons::ICON_CACHE.lock().unwrap();
            let icon = icon_cache.get_png(file_path.to_path_buf())?.to_vec();
            programs.push(Program {
                path,
                name,
                icon,
            });
        }
    }

    Ok(programs)
}

fn get_display_name(executable: &Path) -> anyhow::Result<String> {
    unsafe {
        let executable_path = executable
            .as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect::<Vec<u16>>();

        let version_info_size = {
            let size = GetFileVersionInfoSizeW(PCWSTR::from_raw(executable_path.as_ptr()), None);
            if size == 0 {
                return Err(windows::core::Error::from_win32().into());
            }
            size
        };
        let mut version_info_buf = vec![0u8; version_info_size as usize];
        GetFileVersionInfoW(
            PCWSTR::from_raw(executable_path.as_ptr()),
            0,
            version_info_size,
            version_info_buf.as_mut_ptr() as _,
        )?;

        // this is a pointer to an array of lang/codepage word pairs,
        // but in practice almost all apps only ship with one language.
        // we just treat it as a single thing for simplicity.
        let mut lang_ptr: *const (u16, u16) = std::ptr::null_mut();
        let mut len = 0;

        VerQueryValueW(
            version_info_buf.as_mut_ptr() as _,
            w!("\\VarFileInfo\\Translation"),
            &mut lang_ptr as *const _ as _,
            &mut len,
        )
        .ok()?;
        if len == 0 {
            return Err(anyhow!("no translation info"));
        }

        let sub_block = format!(
            "\\StringFileInfo\\{:04x}{:04x}\\FileDescription\0",
            (*lang_ptr).0,
            (*lang_ptr).1
        )
        .encode_utf16()
        .collect::<Vec<u16>>();
        let mut file_description_ptr: *const u16 = std::ptr::null();
        VerQueryValueW(
            version_info_buf.as_mut_ptr() as _,
            PCWSTR::from_raw(sub_block.as_ptr()),
            &mut file_description_ptr as *const _ as _,
            &mut len,
        )
        .ok()?;
        if len == 0 {
            return Err(anyhow!("no file description"));
        }

        let file_description = std::slice::from_raw_parts(file_description_ptr, len as usize - 1);
        let file_description = String::from_utf16_lossy(file_description);

        Ok(file_description)
    }
}

fn get_foreground_processes() -> Vec<u32> {
    let mut hwnds: Vec<HWND> = Default::default();
    unsafe {
        let _ = EnumWindows(Some(enum_window), LPARAM(&mut hwnds as *mut _ as isize));
    }
    let mut set = HashSet::new();
    for hwnd in hwnds.iter().cloned() {
        let owner = get_owner_window(hwnd);
        let title = get_window_title(owner);
        if is_main_window(owner) && is_visible_window(owner) && !title.is_empty() {
            let pid = get_window_pid(owner);
            set.insert(pid);
        }
    }
    let list: Vec<u32> = set.into_iter().collect();
    list
}

extern "system" fn enum_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows: &mut Vec<HWND> = unsafe { &mut *(lparam.0 as *mut _) };
    windows.push(hwnd);
    BOOL(1)
}

fn get_owner_window(hwnd: HWND) -> HWND {
    unsafe { GetWindow(hwnd, GW_OWNER) }
}

fn is_main_window(hwnd: HWND) -> bool {
    hwnd != HWND(0)
}

fn is_visible_window(hwnd: HWND) -> bool {
    let ret = unsafe { IsWindowVisible(hwnd) };
    ret.as_bool()
}

fn get_window_title(hwnd: HWND) -> String {
    let mut buf = [0u16; 512];
    let len = unsafe { GetWindowTextW(hwnd, buf.as_mut_slice()) };
    if len == 0 {
        return String::new();
    }
    String::from_utf16_lossy(&buf[..len as usize])
}

fn get_window_pid(hwnd: HWND) -> u32 {
    let mut pid: u32 = 0;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid as *mut u32)) };
    pid
}

fn get_program_path_by_hwnd(hwnd: HWND) -> Option<String> {
    let pid = get_window_pid(hwnd);
    get_program_path(pid)
}

fn get_program_path(pid: u32) -> Option<String> {
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
    get_program_path_by_hwnd(hwnd)
}

pub fn get_foreground_program_path() -> Option<String> {
    let hwnd = unsafe { GetForegroundWindow() };
    get_program_path_by_hwnd(hwnd)
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

    let path = get_program_path_by_hwnd(hwnd);
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
        println!("{:#?}", list);
    }

    #[test]
    fn test_app() {
        if let Err(err) = App::start() {
            println!("watcher error: {}", err);
        }
    }

    #[test]
    fn test_get_foreground_processes() {
        let list = get_foreground_processes();
        println!("{:?}", list);
    }
}
