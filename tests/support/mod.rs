#![allow(dead_code)]

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn run_cli(arguments: &[&str]) -> Result<String, String> {
    let mut args = vec!["nova-go".to_string()];
    args.extend(arguments.iter().map(|value| value.to_string()));
    nova_go::run_cli(args).map_err(|error| error.to_string())
}

pub fn write_temp_source(name: &str, contents: &str) -> PathBuf {
    let unique_suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("nova-go-{name}-{unique_suffix}.go"));
    fs::write(&path, contents).expect("temporary source file should be written");
    path
}

pub fn cleanup_temp_source(path: PathBuf) {
    let _ = fs::remove_file(path);
}
