
plugins {
    java
    application
}

repositories {
    mavenCentral()
}

dependencies {
    implementation("com.google.code.gson:gson:2.10.1")
    runtimeOnly("org.junit.jupiter:junit-jupiter:5.10.0")
}

application {
    mainClass.set("com.example.Main")
}
