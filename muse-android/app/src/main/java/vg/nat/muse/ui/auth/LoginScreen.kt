package vg.nat.muse.ui.auth

import android.net.Uri
import androidx.browser.customtabs.CustomTabsIntent
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Cottage
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import kotlinx.coroutines.launch
import vg.nat.muse.net.AuthException
import vg.nat.muse.ui.LocalAuthManager

@Composable
fun LoginScreen() {
    val authManager = LocalAuthManager.current
    val context = LocalContext.current
    val scope = rememberCoroutineScope()

    var serverUrl by remember { mutableStateOf(authManager.serverUrl) }
    var loading by remember { mutableStateOf(false) }
    var error by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(Unit) { if (serverUrl.isEmpty()) serverUrl = authManager.serverUrl }

    Column(
        modifier = Modifier
            .fillMaxSize()
            .padding(horizontal = 32.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.Center,
    ) {
        Icon(
            imageVector = Icons.Rounded.Cottage,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.primary,
            modifier = Modifier.height(72.dp),
        )
        Spacer(Modifier.height(12.dp))
        Text("muse", style = MaterialTheme.typography.headlineLarge)
        Spacer(Modifier.height(4.dp))
        Text(
            "Your personal music server",
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
        Spacer(Modifier.height(40.dp))

        OutlinedTextField(
            value = serverUrl,
            onValueChange = { serverUrl = it },
            label = { Text("Server URL") },
            singleLine = true,
            keyboardOptions = KeyboardOptions(
                keyboardType = KeyboardType.Uri,
                imeAction = ImeAction.Done,
            ),
            modifier = Modifier.fillMaxWidth(),
        )

        error?.let {
            Spacer(Modifier.height(12.dp))
            Text(it, color = MaterialTheme.colorScheme.error, style = MaterialTheme.typography.bodySmall)
        }

        Spacer(Modifier.height(16.dp))
        Button(
            onClick = {
                val trimmed = serverUrl.trim().trimEnd('/')
                if (trimmed.isEmpty()) return@Button
                authManager.serverUrl = trimmed
                loading = true
                error = null
                scope.launch {
                    try {
                        val url = authManager.beginLogin()
                        CustomTabsIntent.Builder().build().launchUrl(context, Uri.parse(url))
                        authManager.awaitCallback()
                    } catch (e: AuthException) {
                        error = if (e is AuthException.LoginFailed ||
                            e is AuthException.NoServerUrl ||
                            e is AuthException.Unauthorized
                        ) e.message else null
                    } catch (e: Exception) {
                        error = e.message
                    } finally {
                        loading = false
                    }
                }
            },
            enabled = !loading && serverUrl.isNotBlank(),
            shape = RoundedCornerShape(12.dp),
            modifier = Modifier
                .fillMaxWidth()
                .height(50.dp),
        ) {
            if (loading) {
                CircularProgressIndicator(
                    strokeWidth = 2.dp,
                    color = MaterialTheme.colorScheme.onPrimary,
                    modifier = Modifier.height(20.dp),
                )
            } else {
                Text(if (loading) "Signing in\u2026" else "Sign In")
            }
        }
    }
}
