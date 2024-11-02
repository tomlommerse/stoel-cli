use ansi_term::Colour;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

// run the differnt fucntions need to deploy the stack
pub fn deploy_swarm(stack_name: &str) -> io::Result<()> {
    let current_dir = env::current_dir()?;
    let files_map = get_yaml_files(&current_dir)?;
    let all_files = list_yaml_files(&files_map, &current_dir)?;
    let selected_files = ask_for_file_selection(&all_files)?;
    
    docker_stack_deploy(stack_name, selected_files)?;

    Ok(())
}

// get all the yaml files in the current directory
fn get_yaml_files(current_dir: &Path) -> io::Result<HashMap<PathBuf, Vec<PathBuf>>> {
    let mut files_map = HashMap::new();
    visit_dirs(current_dir, &mut files_map)?;
    Ok(files_map)
}

// visit all the directories and get the yaml files
fn visit_dirs(dir: &Path, files_map: &mut HashMap<PathBuf, Vec<PathBuf>>) -> io::Result<()> {
    if dir.is_dir() {
        let mut yaml_files: Vec<_> = fs::read_dir(dir)?
            .filter_map(Result::ok)
            .filter(|entry| {
                let path = entry.path();
                path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("yaml")
            })
            .map(|entry| entry.path())
            .collect();
        
        yaml_files.sort();
        files_map.insert(dir.to_path_buf(), yaml_files);

        for entry in fs::read_dir(dir)? {
            let path = entry?.path();
            if path.is_dir() {
                visit_dirs(&path, files_map)?;
            }
        }
    }
    Ok(())
}

// list all the yaml files in the current directory
fn list_yaml_files(files_map: &HashMap<PathBuf, Vec<PathBuf>>, current_dir: &Path) -> io::Result<Vec<String>> {
    let mut all_files = Vec::new();
    let mut file_counter = 1;

    for (dir, files) in sorted_directories(files_map) {
        if !files.is_empty() {
            println!("{}", (dir.display().to_string()));
            for file in files {
                let relative_path = file.strip_prefix(current_dir).unwrap();
                let file_stem = relative_path.file_stem().unwrap().to_str().unwrap();
                println!(
                    " {}. {}",
                    (file_counter.to_string()),
                    Colour::Yellow.paint(file_stem)
                );
                all_files.push(relative_path.display().to_string());
                file_counter += 1;
            }
            println!();
        }
    }
    Ok(all_files)
}

// sort the directories
fn sorted_directories(files_map: &HashMap<PathBuf, Vec<PathBuf>>) -> Vec<(PathBuf, Vec<PathBuf>)> {
    let mut directories: Vec<_> = files_map.iter().collect();
    directories.sort_by_key(|(dir, _)| (*dir).clone());
    
    directories
        .into_iter()
        .map(|(dir, files)| (dir.clone(), files.clone()))
        .collect()
}

// ask the user for the file selection
fn ask_for_file_selection(all_files: &[String]) -> io::Result<HashSet<String>> {
    println!("What containers would you like to add to the stack? (use 0 for all): ");
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim();
    if input == "0" {
        return Ok(all_files.iter().cloned().collect());
    }

    let selected_numbers: HashSet<usize> = input
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();

    let selected_files = selected_numbers
        .iter()
        .filter_map(|&num| all_files.get(num - 1))
        .map(String::from)
        .collect();

    Ok(selected_files)
}

// deploy the stack
fn docker_stack_deploy(stack_name: &str, selected_files: HashSet<String>) -> io::Result<()> {
    let command_str = format!(
        "docker stack deploy {} {} --detach=false --with-registry-auth",
        selected_files.iter().map(|f| format!("-c {}", f)).collect::<Vec<_>>().join(" "),
        stack_name
    );

    println!("Running command: {}", Colour::Blue.paint(&command_str));
    
    let output = Command::new("docker")
        .arg("stack").arg("deploy")
        .args(selected_files.iter().flat_map(|f| vec!["-c", f]))
        .arg(stack_name)
        .arg("--detach=false")
        .arg("--with-registry-auth")
        .output()?;

    if output.status.success() {
        println!("Docker stack deployed successfully.");
    } else {
        let error_message = String::from_utf8_lossy(&output.stderr);
        eprintln!("Failed to deploy stack: {}", error_message);
    }

    Ok(())
}


