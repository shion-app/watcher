use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Program {
    pub path: String,
    pub name: String,
    pub icon: Vec<u8>,
}
