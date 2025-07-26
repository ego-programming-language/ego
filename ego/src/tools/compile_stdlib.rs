use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let stdlib_dir = Path::new("ego/stdlib");
    let output_dir = Path::new("self/stdlib");

    fs::create_dir_all(output_dir).expect("Failed to create egolib dir");

    let ego_files = collect_ego_files(stdlib_dir);

    for ego_file in ego_files {
        let output_path = output_dir
            .join(ego_file.file_stem().unwrap())
            .with_extension("b");

        println!("Compiling {:?}", ego_file);

        let status = Command::new("target/debug/ego")
            .arg("compile")
            .arg(ego_file.to_str().unwrap())
            .arg(output_path.to_str().unwrap())
            .status()
            .expect("Failed to run ego");

        assert!(status.success(), "Compilation failed for {:?}", ego_file);
    }
}

fn collect_ego_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let entries_r = fs::read_dir(dir);
    if let Ok(entries) = entries_r {
        for entry in entries {
            let path = entry.unwrap().path();
            if path.extension().and_then(|s| s.to_str()) == Some("ego") {
                files.push(path);
            }
        }
    }

    files
}
