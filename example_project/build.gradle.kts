
plugins {
    java
    application
    id("com.github.johnrengelman.shadow") version "8.1.1"
}

repositories {
    mavenCentral()
}

dependencies {
    runtimeOnly("org.junit.jupiter:junit-jupiter:5.10.0")
    implementation("com.google.code.gson:gson:2.10.1")
}

application {
    mainClass.set("com.example.Main")
}

tasks {
    // Imposta il nome del JAR finale
    named<com.github.jengelman.gradle.plugins.shadow.tasks.ShadowJar>("shadowJar") {
        archiveBaseName.set("hello-jvm")
        archiveClassifier.set("")
        archiveVersion.set("0.1.0")
    }

    build {
        dependsOn(shadowJar)
    }
}
