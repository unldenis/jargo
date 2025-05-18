use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::Deserialize;
use tracing::{info, error, debug};

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




fn main() {
    // install global tracing subscriber configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let project_folder = "example_project";
    let jargo_path = format!("{}/Jargo.toml", project_folder);
    let toml_content = fs::read_to_string(&jargo_path).expect("Failed to read Jargo.toml");

    let config: JargoToml = toml::from_str(&toml_content).expect("Failed to parse Jargo.toml");

    debug!("Parsed configuration: {:#?}", config);

    gradle::generate_gradle_files(Path::new(project_folder), &config).expect("Failed to generate Gradle files");

    info!("Gradle files generated in '{}/'", project_folder);

    let gradle_dir = gradle::ensure_gradle_wrapper().expect("Failed to set up Gradle wrapper");

    let gradlew = if cfg!(windows) {
        gradle_dir.join("gradlew.bat")
    } else {
        gradle_dir.join("gradlew")
    };

    // Print gradlew path and check existence for debugging
    if !gradlew.exists() {
        error!("gradlew script not found at {}", gradlew.display());
        std::process::exit(1);
    }

    let output = Command::new(&gradlew)
        .args(&["clean", "shadowJar"])
        .current_dir(project_folder)
        .output()
        .expect(&format!(
            "Failed to execute Gradle at {} (exists: {})",
            gradlew.display(),
            gradlew.exists()
        ));

    if output.status.success() {
        info!("Build successful.");
        info!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
        info!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));
    } else {
        error!("Build failed.");
        error!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
        error!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));
    }


    // Execute the jar
    let jar_build_name = format!("{}-{}", config.package.name, config.package.version);
    let output = Command::new("java")
        .args(&["-jar", &format!("build/libs/{}.jar", jar_build_name)])
        .current_dir(project_folder)
        .output()
        .expect("Failed to run the program");


    if output.status.success() {
        info!("Execution successful.");
        info!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
        info!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));
    } else {
        error!("Execution failed.");
        error!("stdout:\n{}", String::from_utf8_lossy(&output.stdout));
        error!("stderr:\n{}", String::from_utf8_lossy(&output.stderr));
    }

}
