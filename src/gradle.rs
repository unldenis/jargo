use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::fs;

use crate::{DependencyDef, DependencyType, JargoToml};
use tracing::{info, error, debug};

pub fn generate_gradle_files(project_dir: &Path, config: &JargoToml) -> std::io::Result<()> {
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
    id("com.github.johnrengelman.shadow") version "8.1.1"
}}

repositories {{
    mavenCentral()
}}

dependencies {{
{deps}}}

application {{
    mainClass.set("{main_class}")
}}

tasks {{
    // Imposta il nome del JAR finale
    named<com.github.jengelman.gradle.plugins.shadow.tasks.ShadowJar>("shadowJar") {{
        archiveBaseName.set("{name}")
        archiveClassifier.set("")
        archiveVersion.set("{version}")
    }}

    build {{
        dependsOn(shadowJar)
    }}
}}
"#,
        deps = deps,
        main_class = config.package.main,
        name = config.package.name,
        version = config.package.version
    );

    fs::write(project_dir.join("build.gradle.kts"), build_gradle)?;

    Ok(())
}


pub fn ensure_gradle_wrapper() -> std::io::Result<PathBuf> {
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
