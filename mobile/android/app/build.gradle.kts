/*
 * Heavily inspired by:
 * - https://kotlinlang.org/docs/gradle.html
 * - https://github.com/bevyengine/bevy/tree/main/examples/mobile
 */

plugins {
    alias(libs.plugins.android.application)
}

java {
    // https://docs.gradle.org/current/userguide/toolchains.html
    toolchain {
        languageVersion = JavaLanguageVersion.of(17)
    }
}

kotlin {
    // https://kotlinlang.org/docs/gradle-compiler-options.html#all-compiler-options
    compilerOptions {
        languageVersion = org.jetbrains.kotlin.gradle.dsl.KotlinVersion.KOTLIN_2_3
        jvmToolchain(17)
    }
}

android {
    namespace = "dev.meinel.slimymist"
    compileSdk = 37

    // https://developer.android.com/reference/tools/gradle-api/9.1/com/android/build/api/dsl/SigningConfig
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
    // https://developer.android.com/reference/tools/gradle-api/9.1/com/android/build/api/dsl/DefaultConfig
    defaultConfig {
        applicationId = "dev.meinel.slimymist"
        // NOTE: `minSdk` is 26 because this is the minimum supported by `bevy_audio`
        minSdk = 26
        targetSdk = 37
        // NOTE: Increase by 1 on each release
        versionCode = 73
        // NOTE: Update with full semantic version on each release
        versionName = "0.31.3"
        // https://developer.android.com/reference/tools/gradle-api/9.1/com/android/build/api/variant/ExternalNativeBuild
        // NOTE: We need this, otherwise libc++_shared.so will not be inserted
        @Suppress("UnstableApiUsage")
        externalNativeBuild {
            cmake {
                arguments("-DANDROID_STL=c++_shared")
            }
        }
        // https://developer.android.com/reference/tools/gradle-api/9.1/com/android/build/api/dsl/Ndk
        ndk {
            abiFilters.addAll(listOf("arm64-v8a", "armeabi-v7a", "x86_64"))
        }
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }
    // https://developer.android.com/reference/tools/gradle-api/9.1/com/android/build/api/dsl/ExternalNativeBuild
    externalNativeBuild {
        cmake {
            path = file("CMakeLists.txt")
        }
    }
    // https://developer.android.com/reference/tools/gradle-api/9.1/com/android/build/api/dsl/BuildType
    buildTypes {
        getByName("release") {
            // https://developer.android.com/topic/performance/app-optimization/enable-app-optimization
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"))
            signingConfig = signingConfigs.getByName("release")
        }
        // https://developer.android.com/build/build-variants#build-types
        create("google") {
            initWith(getByName("release"))
            signingConfig = signingConfigs.getByName("google")
        }
    }
    // https://developer.android.com/reference/tools/gradle-api/9.1/com/android/build/api/dsl/CompileOptions
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    // https://developer.android.com/reference/tools/gradle-api/9.1/com/android/build/api/dsl/BuildFeatures
    buildFeatures {
        prefab = true
    }
    packaging {
        // https://developer.android.com/reference/tools/gradle-api/9.1/com/android/build/api/dsl/JniLibsPackaging
        jniLibs.excludes.add("lib/*/libdummy.so")
        jniLibs.pickFirsts.add("lib/*/libc++_shared.so")
    }
    // https://developer.android.com/reference/tools/gradle-api/9.1/com/android/build/api/dsl/AndroidSourceSet
    sourceSets {
        getByName("main") {
            assets {
                directories += "../../../assets"
            }
        }
    }
}

dependencies {
    implementation(libs.appcompat)
    implementation(libs.core)
    implementation(libs.material)
    implementation(libs.games.activity)
    implementation(libs.core.ktx)
}
