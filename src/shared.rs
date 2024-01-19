use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Program {
    path: String,
    name: String,
    icon: Vec<u8>,
}
