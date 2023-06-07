import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    // Apply the kotlin-jvm plugin for Kotlin JVM projects.
    kotlin("jvm") version "1.6.21"
    // Apply the maven-publish plugin for publishing the library.
    id("maven-publish")
    id("java-library")
    id("signing")
}

group = "io.amplica.graphsdk"
version = "0.0.1-SNAPSHOT"
java.sourceCompatibility = JavaVersion.VERSION_17

repositories {
	maven {
		name = "GithubPackages"
		url = uri("https://maven.pkg.github.com/LibertyDSNP/graph-sdk")
		credentials {
			username = project.findProperty("gpr.user") as String? ?: System.getenv("GITHUB_ACTOR")
			password = project.findProperty("gpr.key") as String? ?: System.getenv("GITHUB_TOKEN")
		}
	}
	mavenLocal()
	mavenCentral()
	maven("https://jitpack.io")
}

dependencies {
    // These dependencies are used only for junit tests
    testImplementation("org.junit.jupiter:junit-jupiter:5.9.1")
    testImplementation("io.github.hakky54:logcaptor:2.9.0")

    implementation("org.slf4j:slf4j-api:2.0.7")
    implementation("com.google.protobuf:protobuf-java:3.23.0")
}

java {
	withSourcesJar()
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
        // Uncomment below to see log output in tests
        // showStandardStreams = true
		events("passed", "skipped", "failed")
	}
}

// Configure publishing
publishing {
    publications {
        create<MavenPublication>("gpr") {
            from(components["java"])
        }
    }
    repositories {
        maven {
            name = "GitHubPackages"
            url = uri("https://maven.pkg.github.com/LibertyDSNP/graph-sdk")
            credentials {
                username = project.findProperty("gpr.user") as String? ?: System.getenv("GITHUB_ACTOR")
                password = project.findProperty("gpr.key") as String? ?: System.getenv("GITHUB_TOKEN")
            }
        }
    }
}
