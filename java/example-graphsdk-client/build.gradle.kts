import org.jetbrains.kotlin.gradle.tasks.KotlinCompile
/*
 * This build file was generated by the Gradle 'init' task.
 *
 * This generated file contains a commented-out sample Kotlin project to get you started.
 * For more details take a look at the Kotlin Quickstart chapter in the Gradle
 * user guide available at https://docs.gradle.org/7.0/userguide/tutorial_kotlin_projects.html
 */

// Apply the Kotlin JVM plugin to add support for Kotlin
plugins {
    // Apply the kotlin-jvm plugin for Kotlin JVM projects.
    kotlin("jvm") version "1.6.21"
    id("java")
}
java.sourceCompatibility = JavaVersion.VERSION_17

// Use the 'repositories' block to declare where to find your dependencies
repositories {
    // Add the GitHub Packages repository as a source for resolving dependencies
    maven {
        name = "GithubPackages"
        url = uri("https://maven.pkg.github.com/LibertyDSNP/graph-sdk")
        credentials {
            username = project.findProperty("gpr.user") as String? ?: System.getenv("GITHUB_ACTOR")
            password = project.findProperty("gpr.key") as String? ?: System.getenv("GITHUB_TOKEN")
        }
    }

    // You can declare any Maven/Ivy/file repository here.
    mavenLocal()
    mavenCentral()
    maven("https://jitpack.io")
}

dependencies {
    implementation("org.slf4j:slf4j-api:2.0.7")
    implementation("org.slf4j:slf4j-simple:2.0.7")
    implementation("io.amplica.graphsdk:lib:1.0.2")
    testImplementation("org.junit.jupiter:junit-jupiter-api:5.8.1")
    testImplementation("org.jetbrains.kotlin:kotlin-scripting-compiler-embeddable:1.6.21")
    testImplementation("org.jetbrains.kotlin:kotlin-script-runtime:1.6.21")
    testRuntimeOnly("org.junit.jupiter:junit-jupiter-engine:5.8.1")
}

tasks.withType<KotlinCompile> {
	kotlinOptions {
		freeCompilerArgs = listOf("-Xjsr305=strict")
		jvmTarget = "17"
	}
}

tasks.withType<Test> {
	useJUnitPlatform()
	testLogging {
		events("passed", "skipped", "failed")
	}
}
