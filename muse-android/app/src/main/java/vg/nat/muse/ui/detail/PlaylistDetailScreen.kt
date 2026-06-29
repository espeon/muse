package vg.nat.muse.ui.detail

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.Shuffle
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FilledTonalButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import vg.nat.muse.net.PlaylistDetail
import vg.nat.muse.ui.FocusableItem
import vg.nat.muse.ui.LocalApiClient
import vg.nat.muse.ui.LocalPlayerEngine
import vg.nat.muse.ui.components.ArtworkImage
import vg.nat.muse.ui.toTracks
import kotlin.random.Random

@Composable
fun PlaylistDetailScreen(
    playlistId: Int,
    contentPadding: PaddingValues = PaddingValues(0.dp),
) {
    val api = LocalApiClient.current
    val player = LocalPlayerEngine.current
    var detail by remember { mutableStateOf<PlaylistDetail?>(null) }
    var loading by remember { mutableStateOf(true) }
    var error by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(playlistId) {
        loading = true
        error = null
        try {
            detail = withContext(Dispatchers.IO) { api.fetchPlaylist(playlistId) }
        } catch (e: Exception) {
            error = e.message
        }
        loading = false
    }

    when {
        loading -> Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            CircularProgressIndicator()
        }
        error != null -> Box(Modifier.fillMaxSize().padding(24.dp), contentAlignment = Alignment.Center) {
            Text(error!!, color = MaterialTheme.colorScheme.error)
        }
        detail != null -> {
            val d = detail!!
            val tracks = d.tracks
            val orientation = LocalConfiguration.current.orientation
            val isLandscape = orientation == android.content.res.Configuration.ORIENTATION_LANDSCAPE
            val firstTrackFocus = remember { FocusRequester() }

            LaunchedEffect(Unit) {
                if (tracks.isNotEmpty()) firstTrackFocus.requestFocus()
            }

            val playlistMeta = @Composable {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(16.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                        FocusableItem(onClick = { player.play(tracks.toTracks(), 0) }) {
                            FilledTonalButton(onClick = { player.play(tracks.toTracks(), 0) }) {
                                Icon(Icons.Rounded.Shuffle, contentDescription = null)
                                Spacer(Modifier.height(0.dp))
                                Text("Shuffle")
                            }
                        }
                        FocusableItem(onClick = { player.play(tracks.toTracks().shuffled(Random), 0) }) {
                            Button(onClick = { player.play(tracks.toTracks().shuffled(Random), 0) }) {
                                Icon(Icons.Rounded.PlayArrow, contentDescription = null)
                                Text("Play")
                            }
                        }
                    }
                }
            }

            if (isLandscape) {
                Row(
                    Modifier
                        .fillMaxSize()
                        .padding(contentPadding),
                    horizontalArrangement = Arrangement.spacedBy(24.dp),
                ) {
                    Column(
                        Modifier
                            .weight(0.8f)
                            .fillMaxSize()
                            .verticalScroll(rememberScrollState()),
                        horizontalAlignment = Alignment.CenterHorizontally,
                        verticalArrangement = Arrangement.Center,
                    ) {
                        playlistMeta()
                    }
                    LazyColumn(
                        Modifier.weight(1f).fillMaxSize(),
                        contentPadding = PaddingValues(vertical = 8.dp),
                    ) {
                        itemsIndexed(tracks, key = { _, t -> t.itemId }) { index, track ->
                            FocusableItem(
                                onClick = {
                                    val list = tracks.toTracks()
                                    player.play(list, index.coerceIn(list.indices))
                                },
                                modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 6.dp),
                                focusRequester = if (index == 0) firstTrackFocus else remember { FocusRequester() },
                            ) {
                                Row(
                                    verticalAlignment = Alignment.CenterVertically,
                                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                                ) {
                                    ArtworkImage(url = track.artUrl, size = 44, cornerRadius = 6)
                                    Column(Modifier.weight(1f)) {
                                        Text(
                                            track.name,
                                            style = MaterialTheme.typography.bodyLarge,
                                            color = MaterialTheme.colorScheme.onSurface,
                                            maxLines = 1,
                                            overflow = TextOverflow.Ellipsis,
                                        )
                                        Text(
                                            track.artistName,
                                            style = MaterialTheme.typography.bodySmall,
                                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                                            maxLines = 1,
                                        )
                                    }
                                    Text(
                                        track.formattedDuration,
                                        style = MaterialTheme.typography.bodySmall,
                                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                                    )
                                }
                            }
                        }
                    }
                }
            } else {
                LazyColumn(Modifier.fillMaxSize(), contentPadding = contentPadding) {
                    item { playlistMeta() }
                    itemsIndexed(tracks, key = { _, t -> t.itemId }) { index, track ->
                        FocusableItem(
                            onClick = {
                                val list = tracks.toTracks()
                                player.play(list, index.coerceIn(list.indices))
                            },
                            modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 6.dp),
                            focusRequester = if (index == 0) firstTrackFocus else remember { FocusRequester() },
                        ) {
                            Row(
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(12.dp),
                            ) {
                                ArtworkImage(url = track.artUrl, size = 44, cornerRadius = 6)
                                Column(Modifier.weight(1f)) {
                                    Text(
                                        track.name,
                                        style = MaterialTheme.typography.bodyLarge,
                                        color = MaterialTheme.colorScheme.onSurface,
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis,
                                    )
                                    Text(
                                        track.artistName,
                                        style = MaterialTheme.typography.bodySmall,
                                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                                        maxLines = 1,
                                    )
                                }
                                Text(
                                    track.formattedDuration,
                                    style = MaterialTheme.typography.bodySmall,
                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                )
                            }
                        }
                    }
                }
            }
        }
    }
}
