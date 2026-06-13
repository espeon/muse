package vg.nat.muse.ui

import androidx.compose.runtime.staticCompositionLocalOf
import vg.nat.muse.net.ApiClient
import vg.nat.muse.net.AuthManager
import vg.nat.muse.player.PlayerEngine

val LocalApiClient = staticCompositionLocalOf<ApiClient> {
    error("ApiClient not provided")
}

val LocalPlayerEngine = staticCompositionLocalOf<PlayerEngine> {
    error("PlayerEngine not provided")
}

val LocalAuthManager = staticCompositionLocalOf<AuthManager> {
    error("AuthManager not provided")
}
