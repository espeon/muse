package vg.nat.muse

import android.app.Application
import vg.nat.muse.net.ApiClient
import vg.nat.muse.net.AuthManager
import vg.nat.muse.player.PlayerEngine

class MuseApplication : Application() {
    lateinit var authManager: AuthManager
        private set

    lateinit var apiClient: ApiClient
        private set

    lateinit var playerEngine: PlayerEngine
        private set

    override fun onCreate() {
        super.onCreate()
        authManager = AuthManager(this)
        apiClient = ApiClient(authManager)
        playerEngine = PlayerEngine(this, apiClient)
    }
}
