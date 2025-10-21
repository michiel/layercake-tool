use layercake;

use std::fs;
use std::path::{Path, PathBuf};

/// Compares the text content of files in two directories.
/// Assumes both directories contain the same filenames.
fn compare_directories(dir1: &Path, dir2: &Path) -> Result<(), String> {
    let path1 = dir1;
    let path2 = dir2;

    if !path1.exists() || !path2.exists() {
        return Err("One or both directories do not exist".to_string());
    }

    let entries1 = fs::read_dir(path1)
        .map_err(|e| format!("Failed to read directory {}: {}", path1.display(), e))?;
    let entries2 = fs::read_dir(path2)
        .map_err(|e| format!("Failed to read directory {}: {}", path2.display(), e))?;

    let mut files1: Vec<_> = entries1
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    let mut files2: Vec<_> = entries2
        .filter_map(Result::ok)
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();

    files1.sort();
    files2.sort();

    if files1 != files2 {
        return Err(format!(
            "Directory contents differ:\n{:?}\n!=\n{:?}",
            files1, files2
        ));
    }

    for file in &files1 {
        let file1_path = path1.join(file);
        let file2_path = path2.join(file);

        let content1 = fs::read_to_string(&file1_path)
            .map_err(|e| format!("Failed to read {}: {}", file1_path.display(), e))?;
        let content2 = fs::read_to_string(&file2_path)
            .map_err(|e| format!("Failed to read {}: {}", file2_path.display(), e))?;

        if content1 != content2 {
            return Err(format!(
                "File contents differ: {}\n\nExpected:\n{}\n\nGot:\n{}",
                file, content1, content2
            ));
        }
    }

    Ok(())
}

#[test]
fn reference_exports() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let plan_path = manifest_dir.join("../sample/ref/plan.yaml");
    let output_dir = manifest_dir.join("out");
    let reference_dir = manifest_dir.join("tests/reference-output");

    if output_dir.exists() {
        fs::remove_dir_all(&output_dir).ok();
    }

    layercake::plan_execution::execute_plan(
        plan_path.clone().to_string_lossy().into_owned(),
        false,
    )
    .unwrap();

    assert!(
        output_dir.exists(),
        "Expected output directory {} to exist after plan execution",
        output_dir.display()
    );

    match compare_directories(&reference_dir, &output_dir) {
        Ok(_) => (),
        Err(e) => panic!("Test failed: {}", e),
    }
}
