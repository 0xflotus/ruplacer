extern crate ruplacer;
extern crate tempdir;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempdir::TempDir;

use ruplacer::query;
use ruplacer::DirectoryPatcher;
use ruplacer::Settings;

fn setup_test(tmp_dir: &TempDir) -> PathBuf {
    let tmp_path = tmp_dir.path();
    let status = Command::new("cp")
        .args(&["-R", "tests/data", &tmp_path.to_string_lossy()])
        .status()
        .expect("Failed to execute process");
    assert!(status.success());
    tmp_path.join("data")
}

fn assert_replaced(path: &Path) {
    let contents =
        fs::read_to_string(&path).unwrap_or_else(|_| panic!("Could not read from {:?}", path));
    assert!(contents.contains("new"));
    assert!(!contents.contains("old"));
}

fn assert_not_replaced(path: &Path) {
    let contents =
        fs::read_to_string(&path).unwrap_or_else(|_| panic!("Could not read from {:?}", path));
    assert!(!contents.contains("new"));
    assert!(contents.contains("old"));
}

fn run_ruplacer(data_path: &Path, settings: Settings) -> DirectoryPatcher {
    let mut directory_patcher = DirectoryPatcher::new(data_path.to_path_buf(), settings);
    directory_patcher
        .patch(&query::substring("old", "new"))
        .expect("ruplacer failed");
    directory_patcher
}

#[test]
fn test_replace_old_by_new() {
    let tmp_dir = TempDir::new("test-ruplacer").expect("failed to create temp dir");
    let data_path = setup_test(&tmp_dir);

    run_ruplacer(&data_path, Settings::default());
    let top_txt_path = data_path.join("top.txt");
    assert_replaced(&top_txt_path);

    // Also check recursion inside the data dir:
    let foo_path = data_path.join("a_dir/sub/foo.txt");
    assert_replaced(&foo_path);
}

#[test]
fn test_stats() {
    let tmp_dir = TempDir::new("test-ruplacer").expect("failed to create temp dir");
    let data_path = setup_test(&tmp_dir);

    let patcher = run_ruplacer(&data_path, Settings::default());
    let stats = patcher.stats();
    assert_eq!(stats.matching_files, 2);
    assert_eq!(stats.num_replacements, 3);
}

#[test]
fn test_dry_run() {
    let tmp_dir = TempDir::new("test-ruplacer").expect("failed to create temp dir");
    let data_path = setup_test(&tmp_dir);

    let mut settings = Settings::default();
    settings.dry_run = true;
    run_ruplacer(&data_path, settings);

    let top_txt_path = data_path.join("top.txt");
    assert_not_replaced(&top_txt_path);
}

#[test]
fn test_with_gitignore() {
    let tmp_dir = TempDir::new("test-ruplacer").expect("failed to create temp dir");
    let data_path = setup_test(&tmp_dir);

    run_ruplacer(&data_path, Settings::default());

    let ignored_path = data_path.join(".hidden/hidden.txt");
    assert_not_replaced(&ignored_path);
}

#[test]
fn test_skip_non_utf8_files() {
    let tmp_dir = TempDir::new("test-ruplacer").expect("failed to create temp dir");
    let data_path = setup_test(&tmp_dir);
    let bin_path = data_path.join("foo.latin1");
    fs::write(bin_path, b"caf\xef\n").unwrap();

    run_ruplacer(&data_path, Settings::default());
}

fn add_python_file(data_path: &Path) -> PathBuf {
    let py_path = data_path.join("foo.py");
    fs::write(&py_path, "a = 'this is old'\n").unwrap();
    py_path.clone()
}

#[test]
fn test_select_file_types() {
    let tmp_dir = TempDir::new("test-ruplacer").expect("failed to create temp dir");
    let data_path = setup_test(&tmp_dir);
    add_python_file(&data_path);
    let mut settings = Settings::default();
    settings.selected_file_types = vec!["py".to_string()];

    let patcher = run_ruplacer(&data_path, settings);

    let stats = patcher.stats();
    assert_eq!(stats.matching_files, 1);
}

#[test]
fn test_ignore_file_types() {
    let tmp_dir = TempDir::new("test-ruplacer").expect("failed to create temp dir");
    let data_path = setup_test(&tmp_dir);
    let py_path = add_python_file(&data_path);
    let mut settings = Settings::default();
    settings.ignored_file_types = vec!["py".to_string()];

    run_ruplacer(&data_path, settings);

    assert_not_replaced(&py_path);
}
