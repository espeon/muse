package vg.nat.muse.net

import android.app.Activity
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import vg.nat.muse.MuseApplication

class AuthRedirectActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val uri: Uri? = intent?.data ?: (intent?.takeIf { it.action == Intent.ACTION_VIEW }?.data)
        val app = applicationContext as? MuseApplication
        if (uri != null && app != null) {
            app.authManager.onRedirect(uri)
        }
        finish()
    }
}
