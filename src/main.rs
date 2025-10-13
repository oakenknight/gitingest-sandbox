use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::Command;

/// CLI for managing the gitingest Docker workflow with security features
#[derive(Parser)]
#[command(
    name = "gitingesters",
    version,
    about = "Run and manage gitingest in Docker with security"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build the gitingest Docker image
    Build {
        /// Force rebuild even if image exists
        #[arg(short, long)]
        force: bool,
    },
    /// Run gitingest against a local repo with security restrictions
    Run {
        /// Path to your code directory
        data_path: String,
        /// Path to store markdown output
        output_path: String,
    },
    /// Run gitingest against a GitHub URL with restricted network access
    RunUrl {
        /// GitHub repository URL
        url: String,
        /// Path to store markdown output
        output_path: String,
    },
    /// Clean up Docker containers and prune data
    Cleanup,
}

const DOCKERFILE_CONTENT: &str = include_str!("../Dockerfile");

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build { force } => {
            build_image(force);
        }
        Commands::Run {
            data_path,
            output_path,
        } => {
            ensure_image_exists();
            run_local_secure(&data_path, &output_path);
            cleanup_containers();
        }
        Commands::RunUrl { url, output_path } => {
            ensure_image_exists();
            run_url_secure(&url, &output_path);
            cleanup_containers();
        }
        Commands::Cleanup => {
            cleanup_containers();
        }
    }
}

fn build_image(force: bool) {
    println!("� Checking Docker image...");

    // Check if image exists
    let image_exists = Command::new("docker")
        .args(["image", "inspect", "gitingest-runner"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    let should_build = if !image_exists {
        println!("   Docker image 'gitingest-runner' not found. Building...");
        true
    } else if force {
        println!("   Force rebuild requested. Rebuilding Docker image...");
        true
    } else {
        println!("   Docker image 'gitingest-runner' already exists.");
        print!("   Rebuild image? (y/N): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes"
    };

    if should_build {
        println!("🛠️  Building Docker image...");

        // Create temporary Dockerfile
        let temp_dockerfile = "Dockerfile.gitingesters";
        if let Err(e) = fs::write(temp_dockerfile, DOCKERFILE_CONTENT) {
            eprintln!("❌ Error creating Dockerfile: {}", e);
            std::process::exit(1);
        }

        let status = Command::new("docker")
            .args([
                "build",
                "-f",
                temp_dockerfile,
                "-t",
                "gitingest-runner",
                ".",
            ])
            .status()
            .expect("failed to run docker build");

        // Clean up temporary Dockerfile
        let _ = fs::remove_file(temp_dockerfile);

        if !status.success() {
            eprintln!("❌ Docker build failed");
            std::process::exit(1);
        }
        println!("✅ Docker image built successfully!");
    }
}

fn ensure_image_exists() {
    let image_exists = Command::new("docker")
        .args(["image", "inspect", "gitingest-runner"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);

    if !image_exists {
        println!("🔧 Docker image not found. Building...");
        build_image(false);
    }
}

fn run_local_secure(data_path: &str, output_path: &str) {
    // Convert to absolute path
    let abs_data_path = Path::new(data_path).canonicalize().unwrap_or_else(|_| {
        eprintln!("❌ Error: Cannot resolve path '{}'", data_path);
        std::process::exit(1);
    });

    let abs_output_path = Path::new(output_path).canonicalize().unwrap_or_else(|_| {
        eprintln!("❌ Error: Cannot resolve path '{}'", output_path);
        std::process::exit(1);
    });

    // Check if data directory exists
    if !abs_data_path.is_dir() {
        eprintln!(
            "❌ Error: Directory '{}' does not exist",
            abs_data_path.display()
        );
        std::process::exit(1);
    }

    println!("🚀 Processing local directory: {}", abs_data_path.display());

    let status = Command::new("docker")
        .args([
            "run",
            "--rm",
            "--network",
            "none",
            "--read-only",
            "--tmpfs",
            "/tmp:rw,noexec,nosuid,size=100m",
            "-v",
            &format!("{}:/data:ro", abs_data_path.display()),
            "-v",
            &format!("{}:/output:rw", abs_output_path.display()),
            "gitingest-runner",
            "/data",
        ])
        .status()
        .expect("failed to run docker container");

    if !status.success() {
        eprintln!("❌ gitingest run failed");
        std::process::exit(1);
    }

    println!("✅ gitingest completed successfully!");
    println!(
        "📄 Secure output saved to: {}/digest.md",
        abs_output_path.display()
    );
}

fn run_url_secure(url: &str, output_path: &str) {
    // Validate GitHub URL
    if !url.starts_with("https://github.com/") {
        eprintln!("❌ Error: Only GitHub URLs are supported for security reasons");
        eprintln!("   URL must start with 'https://github.com/'");
        std::process::exit(1);
    }

    let abs_output_path = Path::new(output_path).canonicalize().unwrap_or_else(|_| {
        eprintln!("❌ Error: Cannot resolve output path '{}'", output_path);
        std::process::exit(1);
    });

    println!("🔒 Running gitingest with RESTRICTED NETWORK ACCESS:");
    println!("🚀 Processing GitHub repository: {}", url);

    let status = Command::new("docker")
        .args([
            "run",
            "--rm",
            "--network",
            "none",
            "--add-host",
            "github.com:140.82.112.3",
            "--add-host",
            "api.github.com:140.82.112.6",
            "--read-only",
            "--tmpfs",
            "/tmp:rw,noexec,nosuid,size=100m",
            "-v",
            &format!("{}:/output:rw", abs_output_path.display()),
            "gitingest-runner",
            url,
        ])
        .status()
        .expect("failed to run docker container");

    if !status.success() {
        eprintln!("❌ gitingest run failed");
        std::process::exit(1);
    }

    println!("✅ gitingest completed successfully!");
    println!(
        "📄 Secure output saved to: {}/digest.md",
        abs_output_path.display()
    );
}

fn cleanup_containers() {
    println!("🧹 Performing security cleanup...");

    // Stop and remove running gitingest containers
    println!("   Stopping any running gitingest containers...");
    let running_containers = Command::new("docker")
        .args([
            "ps",
            "--filter",
            "ancestor=gitingest-runner",
            "--format",
            "{{.ID}}",
        ])
        .output();

    if let Ok(output) = running_containers {
        let container_ids = String::from_utf8_lossy(&output.stdout);
        for container_id in container_ids.lines() {
            if !container_id.trim().is_empty() {
                println!("   Stopping container: {}", container_id.trim());
                let _ = Command::new("docker")
                    .args(["stop", container_id.trim()])
                    .output();
                let _ = Command::new("docker")
                    .args(["rm", container_id.trim()])
                    .output();
            }
        }
    }

    // Remove any orphaned gitingest containers
    println!("   Removing any orphaned gitingest containers...");
    let all_containers = Command::new("docker")
        .args([
            "ps",
            "-a",
            "--filter",
            "ancestor=gitingest-runner",
            "--format",
            "{{.ID}}",
        ])
        .output();

    if let Ok(output) = all_containers {
        let container_ids = String::from_utf8_lossy(&output.stdout);
        for container_id in container_ids.lines() {
            if !container_id.trim().is_empty() {
                println!("   Removing container: {}", container_id.trim());
                let _ = Command::new("docker")
                    .args(["rm", container_id.trim()])
                    .output();
            }
        }
    }

    // Prune unused Docker data
    println!("   Pruning unused Docker data...");
    let _ = Command::new("docker")
        .args(["system", "prune", "-f"])
        .output();

    println!("🔒 Security cleanup completed!");
    println!("   All gitingest containers have been stopped and removed");
    println!("   Unused Docker data has been pruned");
}
