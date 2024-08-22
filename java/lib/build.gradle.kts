import de.undercouch.gradle.tasks.download.Download
import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {
    // Apply the kotlin-jvm plugin for Kotlin JVM projects.
    kotlin("jvm") version "1.6.21"
    // Apply the maven-publish plugin for publishing the library.
    id("maven-publish")
    id("java-library")
    id("signing")
    id("de.undercouch.download") version "5.0.2"
}

group = "io.projectliberty.graphsdk"
val uploadedBinariesVersion = "1.0.1"
java.sourceCompatibility = JavaVersion.VERSION_17
version = if (project.hasProperty("projVersion")) {
    project.properties["projVersion"]!!
} else {
    "1.0.2-SNAPSHOT"
}

repositories {
	maven {
		name = "GithubPackages"
		url = uri("https://maven.pkg.github.com/ProjectLibertyLabs/graph-sdk")
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
    api("com.google.protobuf:protobuf-java:3.23.0")
}

java {
	withSourcesJar()
}

tasks.register("printVersion") {
    shouldRunAfter("build")
    println("version = $version")
}.get()

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

tasks.register("downloadJniBinaries", Download::class.java) {
    enabled = true

    println("Downloading Jni Binaries...")
    val extraResources = arrayOf("dsnp_graph_sdk_jni.dll", "libdsnp_graph_sdk_jni.dylib", "libdsnp_graph_sdk_jni.so")

    src(extraResources.map {
        "https://github.com/ProjectLibertyLabs/graph-sdk/releases/download/v$uploadedBinariesVersion/$it"
    })
    overwrite(true)
    dest("src/main/resources")
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
            url = uri("https://maven.pkg.github.com/ProjectLibertyLabs/graph-sdk")
            credentials {
                username = project.findProperty("gpr.user") as String? ?: System.getenv("GITHUB_ACTOR")
                password = project.findProperty("gpr.key") as String? ?: System.getenv("GITHUB_TOKEN")
            }
        }
    }
}
