use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use serde::Deserialize;
use tracing::{info, error, debug};

#[derive(Debug, Deserialize)]
struct Package {
    name: String,
    version: String,
    main: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum DependencyDef {
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
struct JargoToml {
    package: Package,
    dependencies: Option<HashMap<String, DependencyDef>>,
}

fn generate_gradle_files(project_dir: &Path, config: &JargoToml) -> std::io::Result<()> {
    fs::create_dir_all(project_dir)?;

    let settings = format!(r#"rootProject.name = "{}""#, config.package.name);
    fs::write(project_dir.join("settings.gradle.kts"), settings)?;

    let mut deps = String::new();
    if let Some(dependencies) = &config.dependencies {
        for (_name, dep) in dependencies {
            match dep {
                DependencyDef::Simple(val) => {
                    deps.push_str(&format!("    implementation(\"{}\")\n", val));
                }
                DependencyDef::Full { value, scope } => {
                    let scope_str = match scope {
                        Some(DependencyType::Runtime) => "runtimeOnly",
                        Some(DependencyType::Compile) => "implementation",
                        None => "implementation",
                    };
                    deps.push_str(&format!("    {}(\"{}\")\n", scope_str, value));
                }
            }
        }
    }

    let build_gradle = format!(
        r#"
plugins {{
    java
    application
}}

repositories {{
    mavenCentral()
}}

dependencies {{
{deps}}}

application {{
    mainClass.set("{main_class}")
}}
"#,
        deps = deps,
        main_class = config.package.main
    );

    fs::write(project_dir.join("build.gradle.kts"), build_gradle)?;

    Ok(())
}

fn ensure_gradle_wrapper() -> std::io::Result<PathBuf> {
    use std::io::{Read, Write};

    let home_dir = dirs::home_dir().expect("Could not determine home directory");
    let jargo_dir = home_dir.join(".jargo/gradle-wrapper");

    if !jargo_dir.exists() {
        info!("Gradle wrapper not found, extracting...");

        let zip_data = include_bytes!("../resources/gradle-wrapper.zip");
        let cursor = std::io::Cursor::new(zip_data);
        let mut zip = zip::ZipArchive::new(cursor).expect("Invalid zip");

        for i in 0..zip.len() {
            let mut file = zip.by_index(i).unwrap();
            let outpath = jargo_dir.join(file.name());

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)?;
                continue;
            }

            if let Some(parent) = outpath.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut outfile = fs::File::create(&outpath)?;
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)?;

            // Convert CRLF to LF for text files (gradlew, *.sh, *.bat)
            if cfg!(unix) {
                let name = file.name();
                let is_text_script = name == "gradlew" || name.ends_with(".sh") || name.ends_with(".bat");
                if is_text_script {
                    let text = String::from_utf8_lossy(&contents).replace("\r\n", "\n");
                    outfile.write_all(text.as_bytes())?;
                } else {
                    outfile.write_all(&contents)?;
                }
            } else {
                outfile.write_all(&contents)?;
            }
        }

        info!("Gradle wrapper extracted to {}", jargo_dir.display());
    }

    let gradlew = jargo_dir.join("gradlew");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if gradlew.exists() {
            let perms = fs::Permissions::from_mode(0o755);
            fs::set_permissions(&gradlew, perms)?;
        }
    }

    Ok(jargo_dir)
}


fn main() {
    // install global tracing subscriber configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let project_folder = "example_project";
    let jargo_path = format!("{}/Jargo.toml", project_folder);
    let toml_content = fs::read_to_string(&jargo_path).expect("Failed to read Jargo.toml");

    let config: JargoToml = toml::from_str(&toml_content).expect("Failed to parse Jargo.toml");

    debug!("Parsed configuration: {:#?}", config);

    generate_gradle_files(Path::new(project_folder), &config).expect("Failed to generate Gradle files");

    info!("Gradle files generated in '{}/'", project_folder);

    let gradle_dir = ensure_gradle_wrapper().expect("Failed to set up Gradle wrapper");

    let gradlew = if cfg!(windows) {
        gradle_dir.join("gradlew.bat")
    } else {
        gradle_dir.join("gradlew")
    };

    // Print gradlew path and check existence for debugging
    info!("gradlew path: {}", gradlew.display());
    if !gradlew.exists() {
        error!("gradlew script not found at {}", gradlew.display());
        std::process::exit(1);
    }

    let output = Command::new(&gradlew)
        .args(&["clean", "build"])
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
}
