use std::{process::Command, ffi::OsStr};

use serde::{Deserialize, Serialize};
use windows::{Win32::{Foundation::{HWND, MAX_PATH, SUCCESS, POINT}, UI::WindowsAndMessaging::{GetWindowModuleFileNameW, GetCursorPos, WindowFromPoint, GetForegroundWindow, GetWindowThreadProcessId}, System::Threading::{GetProcessId, PROCESS_QUERY_INFORMATION, OpenProcess, PROCESS_VM_READ, QueryFullProcessImageNameW, PROCESS_NAME_WIN32}}, core::PWSTR};

use crate::Result;

#[derive(Deserialize, Serialize, Debug)]
pub struct Program {
    path: String,
    name: String,
    icon: Vec<u8>
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



mod tests {
    use super::*;

    #[test]
    fn test_get_program_list() {
        let list = get_program_list();
        println!("{:?}", list);
    }
}