use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use clap::Parser;
use serde::Deserialize;
use tracing::{info, error, debug};
use cli::{Cli, Commands};

pub mod gradle;
pub mod cli;

#[derive(Debug, Deserialize)]
pub struct Package {
    name: String,
    version: String,
    main: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum DependencyDef {
    Simple(String),
    Full {
        value: String,
        scope: Option<DependencyType>,
    },
}

#[derive(Debug, Deserialize)]
pub enum DependencyType {
    Compile,
    Runtime,
}

#[derive(Debug, Deserialize)]
pub struct JargoToml {
    package: Package,
    dependencies: Option<HashMap<String, DependencyDef>>,
}


fn create_new_project(project_dir: &Path) -> std::io::Result<()> {
    // fs::create_dir_all(project_dir)?;

    // Crea un Jargo.toml base
    let jargo_toml = r#"
[package]
name = "example_project"
version = "0.1.0"
main = "com.example.Main"

[dependencies]
"#;

    fs::write(project_dir.join("Jargo.toml"), jargo_toml.trim_start())?;


    let git_ignore = r#"
build/
.gradle/
build.gradle.kts
settings.gradle.kts    
"#;

    fs::write(project_dir.join(".gitignore"), git_ignore.trim_start())?;


    // info!("Created Jargo.toml in {}", project_dir.display());

    Ok(())
}


fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let binding = std::env::current_exe().unwrap();
    let exe_path = binding.parent().unwrap();
    let cli = Cli::parse();

    match &cli.command {
        Commands::New { name } => {
            let directory = exe_path.join(name);
            fs::create_dir(&directory)?;

            create_new_project(&directory)?;
            info!("Project created at {}", directory.display());
        }
        Commands::Build { directory_opt } => {
            let mut directory_mut = directory_opt.clone();
            let directory = directory_mut.get_or_insert(exe_path.to_path_buf());
            let jargo_path = directory.join("Jargo.toml");
            let toml_content = std::fs::read_to_string(&jargo_path)?;
            let config: JargoToml = toml::from_str(&toml_content)?;

            gradle::generate_gradle_files(directory, &config)?;
            info!("Gradle files generated.");

            let gradle_dir = gradle::ensure_gradle_wrapper()?;

            let gradlew = if cfg!(windows) {
                gradle_dir.join("gradlew.bat")
            } else {
                gradle_dir.join("gradlew")
            };

            if !gradlew.exists() {
                error!("gradlew not found at {}", gradlew.display());
                std::process::exit(1);
            }

            let output = Command::new(&gradlew)
                .args(&["clean", "shadowJar"])
                .current_dir(directory)
                .output()?;

            if output.status.success() {
                info!("Build successful.");
            } else {
                error!("Build failed.");
            }
        }
    }

    Ok(())
}



// fn main() {
//     // install global tracing subscriber configured based on RUST_LOG env var.
//     tracing_subscriber::fmt::init();

//     let project_folder = "example_project";
//     let jargo_path = format!("{}/Jargo.toml", project_folder);
//     let toml_content = fs::read_to_string(&jargo_path).expect("Failed to read Jargo.toml");

//     let config: JargoToml = toml::from_str(&toml_content).expect("Failed to parse Jargo.toml");

//     debug!("Parsed configuration: {:#?}", config);

//     gradle::generate_gradle_files(Path::new(project_folder), &config).expect("Failed to generate Gradle files");

//     info!("Gradle files generated in '{}/'", project_folder);

//     let gradle_dir = gradle::ensure_gradle_wrapper().expect("Failed to set up Gradle wrapper");

//     let gradlew = if cfg!(windows) {
//         gradle_dir.join("gradlew.bat")
//     } else {
//         gradle_dir.join("gradlew")
//     };

//     // Print gradlew path and check existence for debugging
//     if !gradlew.exists() {
//         error!("gradlew script not found at {}", gradlew.display());
//         std::process::exit(1);
//     }

//     let output = Command::new(&gradlew)
//         .args(&["clean", "shadowJar"])
//         .current_dir(project_folder)
//         .output()
//         .expect(&format!(
//             "Failed to execute Gradle at {} (exists: {})",
//             gradlew.display(),
//             gradlew.exists()
//         ));

//     if output.status.success() {
//         info!("Build successful.");
//         info!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
//         info!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));
//     } else {
//         error!("Build failed.");
//         error!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
//         error!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));
//     }


//     // Execute the jar
//     let jar_build_name = format!("{}-{}", config.package.name, config.package.version);
//     let output = Command::new("java")
//         .args(&["-jar", &format!("build/libs/{}.jar", jar_build_name)])
//         .current_dir(project_folder)
//         .output()
//         .expect("Failed to run the program");


//     if output.status.success() {
//         info!("Execution successful.");
//         info!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
//         info!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));
//     } else {
//         error!("Execution failed.");
//         error!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
//         error!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));
//     }

// }
