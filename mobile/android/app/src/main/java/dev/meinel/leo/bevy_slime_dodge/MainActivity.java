/*
 * File: MainActivity.java
 * Author: Leopold Johannes Meinel (leo@meinel.dev)
 * -----
 * Copyright (c) 2026 Leopold Johannes Meinel & contributors
 * SPDX ID: Apache-2.0
 * URL: https://www.apache.org/licenses/LICENSE-2.0
 * -----
 * Heavily inspired by:
 * - https://github.com/bevyengine/bevy/tree/main/examples/mobile/android_example
 * - https://developer.android.com/develop/ui/views/layout/immersive#java
 */

package dev.meinel.slimymist;

import android.view.View;

import com.google.androidgamesdk.GameActivity;

/**
 * Loads the rust library and handles android specific to integrate with it.
 *
 * The library is loaded at class initialization and provided by jniLibs.
 */
public class MainActivity extends GameActivity {
    // Load rust library
    static {
        System.loadLibrary("slimy_mist");
    }

    /**
     * Called when the current Window of the activity gains or loses focus.
     *
     * We are overriding the default implementation of this in Activity.
     *
     * This just hides the system UI if the app window is focused.
     *
     * @see https://developer.android.com/reference/android/app/Activity#onWindowFocusChanged(boolean)
     */
    @Override
    public void onWindowFocusChanged(boolean hasFocus) {
        // Call parent class implementation of onWindowFocusChanged to make sure that we are updating correctly.
        super.onWindowFocusChanged(hasFocus);

        // If the window has focus, hide system UI.
        if (hasFocus) {
            hideSystemUi();
        }
    }

    /**
     * Hides system UI.
     *
     * This will make the app content fill the entire screen.
     *
     * @see https://developer.android.com/develop/ui/views/layout/immersive#java
     */
    private void hideSystemUi() {
        // https://developer.android.com/reference/android/view/WindowInsetsController
        WindowInsetsControllerCompat windowInsetsController = WindowCompat.getInsetsController(getWindow(), getWindow().getDecorView());

        // Show bars if swiping
        windowInsetsController.setSystemBarsBehavior(
            WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE
        );
        // Hide both the status bar and the navigation bar.
        windowInsetsController.hide(WindowInsetsCompat.Type.systemBars());
    }
}
