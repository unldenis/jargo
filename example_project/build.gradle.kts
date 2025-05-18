
plugins {
    java
    application
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
