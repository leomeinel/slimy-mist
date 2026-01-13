/*
 * File: build.gradle.kts
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * - https://kotlinlang.org/docs/gradle.html
 * - https://github.com/bevyengine/bevy/tree/main/examples/mobile/android_example
 */

plugins {
    alias(libs.plugins.android.application)
}

// Apply a specific Java toolchain to ease working on different environments.
java {
    toolchain {
        languageVersion = JavaLanguageVersion.of(17)
    }
}


android {
    namespace = "dev.meinel.leo.bevy_slime_dodge"
    compileSdk = 34

    defaultConfig {
        applicationId = "dev.meinel.leo.bevy_slime_dodge"
        minSdk = 31
        targetSdk = 33
        versionCode = 1
        versionName = "1.0"
        // We need this, otherwise libc++_shared.so will not be inserted
        externalNativeBuild {
            cmake {
                arguments("-DANDROID_STL=c++_shared")
            }
        }
        // Set up targets
        ndk {
            abiFilters.addAll(listOf("arm64-v8a", "armeabi-v7a", "x86_64"))
        }
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }
    externalNativeBuild {
        cmake {
            path = file("CMakeLists.txt")
        }
    }
    buildTypes {
        getByName("release") {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }
    buildFeatures {
        prefab = true
    }
    packaging {
        jniLibs.excludes.add("lib/*/libdummy.so")
    }
    sourceSets {
        getByName("main") {
            assets {
                srcDir("../../../assets")
            }
            res {
                srcDir("../../../assets/android-res")
            }
        }
    }
}

dependencies {
    implementation(libs.appcompat)
    implementation(libs.material)
    implementation(libs.games.activity)
    testImplementation(libs.junit)
    androidTestImplementation(libs.ext.junit)
    androidTestImplementation(libs.espresso.core)
}
