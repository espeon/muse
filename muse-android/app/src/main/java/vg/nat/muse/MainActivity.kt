package vg.nat.muse

import android.Manifest
import android.content.pm.PackageManager
import android.os.Build
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.core.content.ContextCompat
import vg.nat.muse.ui.LocalApiClient
import vg.nat.muse.ui.LocalAuthManager
import vg.nat.muse.ui.LocalPlayerEngine
import vg.nat.muse.ui.RootScaffold
import vg.nat.muse.ui.auth.LoginScreen
import vg.nat.muse.ui.theme.MuseTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        val app = application as MuseApplication
        setContent {
            MuseTheme {
                CompositionLocalProvider(
                    LocalApiClient provides app.apiClient,
                    LocalPlayerEngine provides app.playerEngine,
                    LocalAuthManager provides app.authManager,
                ) {
                    val authenticated by app.authManager.isAuthenticated.collectAsState()

                    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
                        val launcher = rememberLauncherForActivityResult(
                            ActivityResultContracts.RequestPermission(),
                        ) { /* notification is optional */ }
                        LaunchedEffect(authenticated) {
                            if (authenticated &&
                                ContextCompat.checkSelfPermission(
                                    this@MainActivity, Manifest.permission.POST_NOTIFICATIONS,
                                ) != PackageManager.PERMISSION_GRANTED
                            ) {
                                launcher.launch(Manifest.permission.POST_NOTIFICATIONS)
                            }
                        }
                    }

                    if (authenticated) RootScaffold() else LoginScreen()
                }
            }
        }
    }
}
