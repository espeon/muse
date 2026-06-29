package vg.nat.muse.ui

import androidx.compose.runtime.staticCompositionLocalOf
import vg.nat.muse.net.ApiClient
import vg.nat.muse.net.AuthManager
import vg.nat.muse.net.UmiClient
import vg.nat.muse.lyrics.TranslationService
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

val LocalUmiClient = staticCompositionLocalOf<UmiClient> {
    error("UmiClient not provided")
}

val LocalHasGamepad = staticCompositionLocalOf { false }

val LocalTranslationService = staticCompositionLocalOf<TranslationService> {
    error("TranslationService not provided")
}
