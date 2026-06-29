package vg.nat.muse.ui.settings

import android.net.Uri
import androidx.browser.customtabs.CustomTabsIntent
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
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
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import vg.nat.muse.ui.LocalApiClient
import vg.nat.muse.ui.LocalAuthManager
import vg.nat.muse.ui.LocalPlayerEngine
import vg.nat.muse.ui.LocalTranslationService

private enum class LastfmStep { Idle, AwaitingApproval, Connecting }

@Composable
fun SettingsScreen(contentPadding: PaddingValues = PaddingValues(0.dp)) {
    val authManager = LocalAuthManager.current
    val player = LocalPlayerEngine.current
    val translationService = LocalTranslationService.current
    val apiClient = LocalApiClient.current
    val context = LocalContext.current
    val scope = rememberCoroutineScope()

    val authenticated by authManager.isAuthenticated.collectAsState()
    val hlsProfiles by player.hlsProfiles.collectAsState()

    var serverUrl by remember { mutableStateOf(authManager.serverUrl) }
    var showLogout by remember { mutableStateOf(false) }
    var qualityExpanded by remember { mutableStateOf(false) }
    var llmEndpoint by remember { mutableStateOf(translationService.endpoint) }
    var llmApiKey by remember { mutableStateOf(translationService.apiKey) }
    var llmModel by remember { mutableStateOf(translationService.model) }

    var lastfmConnected by remember { mutableStateOf(false) }
    var lastfmUsername by remember { mutableStateOf<String?>(null) }
    var pendingToken by remember { mutableStateOf<String?>(null) }
    var lastfmStep by remember { mutableStateOf(LastfmStep.Idle) }
    var lastfmError by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(Unit) {
        runCatching { apiClient.fetchMe() }
            .onSuccess { lastfmConnected = it.lastfmConnected }
    }

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
            .padding(contentPadding)
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        Text("Server", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurface)
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

        Text("Account", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurface)
        Text(if (authenticated) "Authenticated" else "Not authenticated", color = MaterialTheme.colorScheme.onSurface)
        if (authenticated) {
            Button(onClick = { showLogout = true }) { Text("Log out") }
        }

        HorizontalDivider()

        Text("Last.fm", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurface)
        when (lastfmStep) {
            LastfmStep.Idle -> {
                if (lastfmConnected) {
                    Text(
                        buildString {
                            append("Connected")
                            lastfmUsername?.let { append(" as $it") }
                        },
                        color = MaterialTheme.colorScheme.onSurface,
                    )
                    Button(onClick = {
                        scope.launch {
                            runCatching { apiClient.disconnectLastfm() }
                                .onSuccess {
                                    lastfmConnected = false
                                    lastfmUsername = null
                                }
                                .onFailure { lastfmError = it.message }
                        }
                    }) { Text("Disconnect") }
                } else {
                    Button(onClick = {
                        scope.launch {
                            lastfmError = null
                            runCatching { apiClient.fetchLastfmToken() }
                                .onSuccess { resp ->
                                    pendingToken = resp.token
                                    lastfmStep = LastfmStep.AwaitingApproval
                                    CustomTabsIntent.Builder().build()
                                        .launchUrl(context, Uri.parse(resp.url))
                                }
                                .onFailure { lastfmError = it.message }
                        }
                    }) { Text("Connect to Last.fm") }
                }
            }
            LastfmStep.AwaitingApproval -> {
                Text(
                    "Complete the authorization in your browser, then return here and tap Continue.",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    Button(onClick = {
                        val token = pendingToken
                        if (token == null) {
                            lastfmStep = LastfmStep.Idle
                            return@Button
                        }
                        lastfmStep = LastfmStep.Connecting
                        scope.launch {
                            runCatching { apiClient.completeLastfmSession(token) }
                                .onSuccess { resp ->
                                    lastfmConnected = true
                                    lastfmUsername = resp.username
                                    pendingToken = null
                                    lastfmStep = LastfmStep.Idle
                                }
                                .onFailure {
                                    lastfmError = it.message
                                    lastfmStep = LastfmStep.AwaitingApproval
                                }
                        }
                    }) { Text("Continue") }
                    TextButton(onClick = {
                        pendingToken = null
                        lastfmStep = LastfmStep.Idle
                    }) { Text("Cancel") }
                }
            }
            LastfmStep.Connecting -> {
                Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    CircularProgressIndicator(modifier = Modifier.size(16.dp), strokeWidth = 2.dp)
                    Text("Connecting\u2026")
                }
            }
        }
        lastfmError?.let {
            Text(it, color = MaterialTheme.colorScheme.error, style = MaterialTheme.typography.bodySmall)
        }

        HorizontalDivider()

        Text("Playback", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurface)
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
            Column(Modifier.weight(1f)) {
                Text("Adaptive Streaming", color = MaterialTheme.colorScheme.onSurface)
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
                Text("Quality", color = MaterialTheme.colorScheme.onSurface)
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

        Text("Translation", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurface)

        var backendExpanded by remember { mutableStateOf(false) }
        Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween, verticalAlignment = Alignment.CenterVertically) {
            Column(Modifier.weight(1f)) {
                Text("Engine", color = MaterialTheme.colorScheme.onSurface)
                Text(
                    when (translationService.backend) {
                        vg.nat.muse.lyrics.TranslationBackend.MLKIT -> "On-device (ML Kit)"
                        vg.nat.muse.lyrics.TranslationBackend.LLM -> "LLM (OpenRouter / OpenAI)"
                    },
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Box {
                TextButton(onClick = { backendExpanded = true }) {
                    Text(when (translationService.backend) {
                        vg.nat.muse.lyrics.TranslationBackend.MLKIT -> "ML Kit"
                        vg.nat.muse.lyrics.TranslationBackend.LLM -> "LLM"
                    })
                }
                DropdownMenu(expanded = backendExpanded, onDismissRequest = { backendExpanded = false }) {
                    DropdownMenuItem(
                        text = { Text("ML Kit (on-device)") },
                        onClick = { translationService.backend = vg.nat.muse.lyrics.TranslationBackend.MLKIT; backendExpanded = false },
                    )
                    DropdownMenuItem(
                        text = { Text("LLM (API)") },
                        onClick = { translationService.backend = vg.nat.muse.lyrics.TranslationBackend.LLM; backendExpanded = false },
                    )
                }
            }
        }

        if (translationService.backend == vg.nat.muse.lyrics.TranslationBackend.LLM) {
            OutlinedTextField(
                value = llmEndpoint,
                onValueChange = {
                    llmEndpoint = it
                    translationService.endpoint = it.trim()
                },
                label = { Text("API Endpoint") },
                placeholder = { Text("https://openrouter.ai/api/v1/chat/completions") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
            OutlinedTextField(
                value = llmApiKey,
                onValueChange = {
                    llmApiKey = it
                    translationService.apiKey = it.trim()
                },
                label = { Text("API Key") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
            OutlinedTextField(
                value = llmModel,
                onValueChange = {
                    llmModel = it
                    translationService.model = it.trim()
                },
                label = { Text("Model") },
                placeholder = { Text("openai/gpt-4o-mini") },
                singleLine = true,
                modifier = Modifier.fillMaxWidth(),
            )
            if (translationService.isConfigured) {
                Text(
                    "Translation is ready",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.primary,
                )
            }
        }

        HorizontalDivider()

        Text("About", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.SemiBold, color = MaterialTheme.colorScheme.onSurface)
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
