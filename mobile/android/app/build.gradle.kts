/*
 * File: build.gradle.kts
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * - https://kotlinlang.org/docs/gradle.html
 * - https://github.com/bevyengine/bevy/tree/main/examples/mobile
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
    namespace = "dev.meinel.slimymist"
    compileSdk = 36

    // https://developer.android.com/reference/tools/gradle-api/8.13/com/android/build/api/dsl/SigningConfig
    signingConfigs {
        create("release") {
            storeFile = file("prod-sign.keystore")
            storePassword = System.getenv("PROD_SIGN_KEYSTORE_PASS")
            keyAlias = System.getenv("PROD_SIGN_KEY_ALIAS")
            keyPassword = System.getenv("PROD_SIGN_KEYSTORE_PASS")
        }
        create("google") {
            storeFile = file("prod-upload.keystore")
            storePassword = System.getenv("PROD_UPLOAD_KEYSTORE_PASS")
            keyAlias = System.getenv("PROD_UPLOAD_KEY_ALIAS")
            keyPassword = System.getenv("PROD_UPLOAD_KEYSTORE_PASS")
        }
    }
    // https://developer.android.com/reference/tools/gradle-api/8.13/com/android/build/api/dsl/DefaultConfig
    defaultConfig {
        applicationId = "dev.meinel.slimymist"
        minSdk = 31
        targetSdk = 36
        // NOTE: Increase by 1 on each release after publishing
        versionCode = 1
        // NOTE: Update with full semantic version on each release
        versionName = "0.11.5"
        // https://developer.android.com/reference/tools/gradle-api/8.13/com/android/build/api/variant/ExternalNativeBuild
        // NOTE: We need this, otherwise libc++_shared.so will not be inserted
        externalNativeBuild {
            cmake {
                arguments("-DANDROID_STL=c++_shared")
            }
        }
        // https://developer.android.com/reference/tools/gradle-api/8.13/com/android/build/api/dsl/Ndk
        ndk {
            abiFilters.addAll(listOf("arm64-v8a", "armeabi-v7a", "x86_64"))
        }
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }
    // https://developer.android.com/reference/tools/gradle-api/8.13/com/android/build/api/dsl/ExternalNativeBuild
    externalNativeBuild {
        cmake {
            path = file("CMakeLists.txt")
        }
    }
    // https://developer.android.com/reference/tools/gradle-api/8.13/com/android/build/api/dsl/BuildType
    buildTypes {
        getByName("release") {
            // https://developer.android.com/topic/performance/app-optimization/enable-app-optimization
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
            signingConfig = signingConfigs.getByName("release")
        }
        // https://developer.android.com/build/build-variants#build-types
        create("google") {
            initWith(getByName("release"))
            signingConfig = signingConfigs.getByName("google")
        }
    }
    // https://developer.android.com/reference/tools/gradle-api/8.13/com/android/build/api/dsl/CompileOptions
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }
    // https://developer.android.com/reference/tools/gradle-api/8.13/com/android/build/api/dsl/BuildFeatures
    buildFeatures {
        prefab = true
    }
    packaging {
        // https://developer.android.com/reference/tools/gradle-api/8.13/com/android/build/api/dsl/JniLibsPackaging
        jniLibs.excludes.add("lib/*/libdummy.so")
        jniLibs.pickFirsts.add("lib/*/libc++_shared.so")
    }
    // https://developer.android.com/reference/tools/gradle-api/8.13/com/android/build/api/dsl/AndroidSourceSet
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
    implementation(libs.core)
    implementation(libs.material)
    implementation(libs.games.activity)
}
