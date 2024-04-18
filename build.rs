const COMMANDS: &[&str] = &["get_program_list", "suspend", "resume", "is_active"];

fn main() {
  tauri_plugin::Builder::new(COMMANDS)
    .android_path("android")
    .ios_path("ios")
    .build();
}
