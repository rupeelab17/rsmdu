use std::path::PathBuf;

pub const TEMP_PATH: &str = "./temp";

pub fn get_temp_path() -> PathBuf {
    PathBuf::from(TEMP_PATH)
}

