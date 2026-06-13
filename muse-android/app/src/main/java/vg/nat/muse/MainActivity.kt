package vg.nat.muse

import android.net.Uri
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.browser.customtabs.CustomTabsIntent
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import vg.nat.muse.net.AuthManager
import vg.nat.muse.ui.theme.MuseTheme

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        val app = application as MuseApplication
        setContent {
            MuseTheme {
                Surface(modifier = Modifier.fillMaxSize()) {
                    AuthGateway(app.authManager)
                }
            }
        }
    }
}

@Composable
private fun AuthGateway(authManager: AuthManager) {
    val context = LocalContext.current
    val scope = rememberCoroutineScope()
    val authenticated by authManager.isAuthenticated.collectAsState()
    var serverUrl by remember { mutableStateOf(authManager.serverUrl) }
    var status by remember { mutableStateOf<String?>(null) }
    var busy by remember { mutableStateOf(false) }

    Column(
        modifier = Modifier.fillMaxSize().padding(24.dp),
        verticalArrangement = Arrangement.Center,
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        if (authenticated) {
            Text("Authenticated", style = MaterialTheme.typography.headlineSmall)
            Spacer(Modifier.height(16.dp))
            Button(onClick = { authManager.logout() }) { Text("Log out") }
        } else {
            OutlinedTextField(
                value = serverUrl,
                onValueChange = { serverUrl = it },
                label = { Text("Server URL") },
                singleLine = true,
            )
            Spacer(Modifier.height(16.dp))
            Button(
                enabled = !busy && serverUrl.isNotBlank(),
                onClick = {
                    authManager.serverUrl = serverUrl.trimEnd('/')
                    busy = true
                    status = null
                    scope.launch {
                        try {
                            val url = authManager.beginLogin()
                            CustomTabsIntent.Builder().build().launchUrl(context, Uri.parse(url))
                            authManager.awaitCallback()
                            status = "Logged in"
                        } catch (e: Exception) {
                            status = e.message ?: e.javaClass.simpleName
                        } finally {
                            busy = false
                        }
                    }
                },
            ) {
                Text(if (busy) "Waiting\u2026" else "Log in")
            }
            status?.let {
                Spacer(Modifier.height(16.dp))
                Text(it, color = MaterialTheme.colorScheme.error)
            }
        }
    }
}
