use std::path::PathBuf;
use std::fs;

/// Return the content of a fixture file as a string
pub fn fixture_content(filename: &str) -> String {
    fs::read_to_string(fixture_filename(filename)).unwrap()
}

/// Return the path to our test data directory
pub fn fixture_dir() -> PathBuf {
    let mut dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    dir.push("fixture");
    dir
}

/// Return the path to a file within the test data directory
pub fn fixture_filename(filename: &str) -> String {
    let mut dir = fixture_dir();
    dir.push(filename);
    dir.to_str().unwrap().to_string()
}
