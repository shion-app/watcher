use std::{process::Command, ffi::OsStr};

use serde::{Deserialize, Serialize};

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

mod tests {
    use super::*;

    #[test]
    fn test_get_program_list() {
        let list = get_program_list();
        println!("{:?}", list);
    }
}