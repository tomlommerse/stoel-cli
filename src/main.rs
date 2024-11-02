use std::env;

mod docker; // Ensure your docker module is declared
mod kipper; // Declare the kipper module

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage(&args[0]);
        return;
    }

    // Get the command and possible stack name
    let command = &args[1];
    let stack_name = args.get(2).map(|s| s.as_str());

    match command.as_str() {
        "deploy" => {
            if let Some(name) = stack_name {
                handle_deploy(name);
            } else {
                eprintln!("Error: Missing stack name for 'deploy' command.");
            }
        }
        "kipper" => kipper::kipper::kipper(),

        _ => print_usage(&args[0]),
    }
}

// Handles the deployment of a Docker stack
fn handle_deploy(stack_name: &str) {
    if let Err(e) = docker::deploy_stack::deploy_swarm(stack_name) {
        eprintln!("Error during deployment: {}", e);
    } else {
        println!("Done");
    }
}

// Function to print usage instructions
fn print_usage(program_name: &str) {
    eprintln!("Usage: {} deploy <stackname> | hello | kipper | 2 | 3", program_name);
}
