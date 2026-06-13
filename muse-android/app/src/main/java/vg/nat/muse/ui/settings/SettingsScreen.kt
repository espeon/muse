package vg.nat.muse.ui.settings

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.DropdownMenu
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import vg.nat.muse.ui.LocalAuthManager
import vg.nat.muse.ui.LocalPlayerEngine

@Composable
fun SettingsScreen() {
    val authManager = LocalAuthManager.current
    val player = LocalPlayerEngine.current
    val context = LocalContext.current

    val authenticated by authManager.isAuthenticated.collectAsState()
    val hlsProfiles by player.hlsProfiles.collectAsState()

    var serverUrl by remember { mutableStateOf(authManager.serverUrl) }
    var showLogout by remember { mutableStateOf(false) }
    var qualityExpanded by remember { mutableStateOf(false) }

    LaunchedEffect(Unit) { player.loadHlsProfiles() }

    val version = remember {
        runCatching {
            val info = context.packageManager.getPackageInfo(context.packageName, 0)
            "${info.versionName} (${info.longVersionCode})"
        }.getOrDefault("unknown")
    }

    Column(
        Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        Text("Server", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold)
        OutlinedTextField(
            value = serverUrl,
            onValueChange = {
                serverUrl = it
                authManager.serverUrl = it.trim().trimEnd('/')
            },
            label = { Text("Server URL") },
            singleLine = true,
            modifier = Modifier.fillMaxWidth(),
        )
        Text(
            "The URL of your Maki server.",
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )

        HorizontalDivider()

        Text("Account", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold)
        Text(if (authenticated) "Authenticated" else "Not authenticated")
        if (authenticated) {
            Button(onClick = { showLogout = true }) { Text("Log out") }
        }

        HorizontalDivider()

        Text("Playback", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold)
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Column(Modifier.weight(1f)) {
                Text("Adaptive Streaming")
                Text(
                    "Auto-adjusts quality. Takes effect on next track.",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Switch(checked = player.useHls, onCheckedChange = { player.useHls = it })
        }
        if (player.useHls) {
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                Text("Quality")
                Box {
                    TextButton(onClick = { qualityExpanded = true }) {
                        Text(player.selectedProfile ?: "Auto")
                    }
                    DropdownMenu(expanded = qualityExpanded, onDismissRequest = { qualityExpanded = false }) {
                        DropdownMenuItem(
                            text = { Text("Auto") },
                            onClick = { player.setQuality(null); qualityExpanded = false },
                        )
                        hlsProfiles.forEach { profile ->
                            DropdownMenuItem(
                                text = { Text(profile.displayName) },
                                onClick = { player.setQuality(profile.name); qualityExpanded = false },
                            )
                        }
                    }
                }
            }
        }

        HorizontalDivider()

        Text("About", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold)
        Text("Version: $version", style = MaterialTheme.typography.bodySmall)
    }

    if (showLogout) {
        AlertDialog(
            onDismissRequest = { showLogout = false },
            title = { Text("Log out") },
            text = { Text("You'll need to sign in again to access your music.") },
            confirmButton = {
                TextButton(onClick = {
                    authManager.logout()
                    showLogout = false
                }) { Text("Log out") }
            },
            dismissButton = { TextButton(onClick = { showLogout = false }) { Text("Cancel") } },
        )
    }
}
