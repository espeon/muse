package vg.nat.muse

import android.app.Application
import android.content.ComponentName
import androidx.core.content.ContextCompat
import androidx.media3.session.MediaController
import androidx.media3.session.SessionToken
import vg.nat.muse.net.ApiClient
import vg.nat.muse.net.AuthManager
import vg.nat.muse.net.UmiClient
import vg.nat.muse.player.PlaybackService
import vg.nat.muse.player.PlayerEngine

class MuseApplication : Application() {
    lateinit var authManager: AuthManager
        private set

    lateinit var apiClient: ApiClient
        private set

    lateinit var umiClient: UmiClient
        private set

    lateinit var playerEngine: PlayerEngine
        private set

    private var mediaController: MediaController? = null

    override fun onCreate() {
        super.onCreate()
        authManager = AuthManager(this)
        apiClient = ApiClient(authManager)
        umiClient = UmiClient { authManager.umiUrl }
        playerEngine = PlayerEngine(this, apiClient)
        connectMediaController()
    }

    private fun connectMediaController() {
        val sessionToken = SessionToken(this, ComponentName(this, PlaybackService::class.java))
        val future = MediaController.Builder(this, sessionToken).buildAsync()
        future.addListener({
            runCatching { mediaController = future.get() }
        }, ContextCompat.getMainExecutor(this))
    }
}
