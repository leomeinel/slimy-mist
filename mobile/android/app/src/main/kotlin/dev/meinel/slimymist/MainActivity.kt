/*
 * File: MainActivity.kt
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by:
 * - https://github.com/bevyengine/bevy/tree/main/examples/mobile
 * - https://developer.android.com/develop/ui/views/layout/immersive#java
 */

package dev.meinel.slimymist

import androidx.core.view.WindowCompat
import androidx.core.view.WindowInsetsCompat
import androidx.core.view.WindowInsetsControllerCompat
import com.google.androidgamesdk.GameActivity

/**
 * Loads the rust library and handles android specific to integrate with it.
 *
 *
 * The library is loaded at class initialization and provided by jniLibs.
 */
class MainActivity : GameActivity() {
    /**
     * Called when the current Window of the activity gains or loses focus.
     *
     *
     * This just hides the system UI if the app window is focused.
     */
    override fun onWindowFocusChanged(hasFocus: Boolean) {
        // Call parent class implementation of onWindowFocusChanged to make sure that we are updating correctly.
        super.onWindowFocusChanged(hasFocus)

        // If the window has focus, hide system UI.
        if (hasFocus) {
            hideSystemUi()
        }
    }

    /**
     * Hides system UI.
     *
     *
     * This will make the app content fill the entire screen.
     */
    private fun hideSystemUi() {
        val windowInsetsController =
            WindowCompat.getInsetsController(window, window.decorView)

        // Show bars if swiping
        windowInsetsController.systemBarsBehavior = WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
        // Hide both the status bar and the navigation bar.
        windowInsetsController.hide(WindowInsetsCompat.Type.systemBars())
    }

    companion object {
        // Load rust library
        init {
            System.loadLibrary("slimy_mist")
        }
    }
}
