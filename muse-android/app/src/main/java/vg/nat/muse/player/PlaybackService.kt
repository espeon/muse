package vg.nat.muse.player

import android.content.Intent
import androidx.media3.session.MediaSession
import androidx.media3.session.MediaSessionService
import vg.nat.muse.MuseApplication

class PlaybackService : MediaSessionService() {

    private var session: MediaSession? = null

    override fun onCreate() {
        super.onCreate()
        val app = applicationContext as MuseApplication
        session = MediaSession.Builder(this, app.playerEngine.player).build()
    }

    override fun onTaskRemoved(rootIntent: Intent?) {
        val player = session?.player
        if (player == null || !player.playWhenReady || player.mediaItemCount == 0) {
            stopSelf()
        }
    }

    override fun onGetSession(controllerInfo: MediaSession.ControllerInfo): MediaSession? = session

    override fun onDestroy() {
        session?.release()
        session = null
        super.onDestroy()
    }
}
