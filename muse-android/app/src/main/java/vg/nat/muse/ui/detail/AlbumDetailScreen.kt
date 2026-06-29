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
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
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
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import vg.nat.muse.net.Album
import vg.nat.muse.net.ApiException
import vg.nat.muse.net.Track
import vg.nat.muse.ui.FocusableItem
import vg.nat.muse.ui.LocalApiClient
import vg.nat.muse.ui.LocalPlayerEngine
import vg.nat.muse.ui.components.ArtworkImage
import vg.nat.muse.ui.components.TrackRow
import kotlin.random.Random

@Composable
fun AlbumDetailScreen(
    albumId: Int,
    onOpenArtist: (Int) -> Unit,
    contentPadding: PaddingValues = PaddingValues(0.dp),
) {
    val api = LocalApiClient.current
    val player = LocalPlayerEngine.current
    var album by remember { mutableStateOf<Album?>(null) }
    var loading by remember { mutableStateOf(true) }
    var error by remember { mutableStateOf<String?>(null) }
    val liked = remember { mutableStateOf(setOf<Int>()) }

    LaunchedEffect(albumId) {
        loading = true
        error = null
        try {
            val fetched = withContext(Dispatchers.IO) { api.fetchAlbum(albumId) }
            album = fetched
            liked.value = fetched.tracks
                ?.filter { it.liked == true }
                ?.map { it.id }
                ?.toSet() ?: emptySet()
        } catch (e: ApiException) {
            error = e.message
        } catch (e: Exception) {
            error = e.message
        }
        loading = false
    }

    val queue by player.queue.collectAsState()
    val idx by player.currentIndex.collectAsState()
    val isPlaying by player.isPlaying.collectAsState()
    val playingId = if (isPlaying) queue.getOrNull(idx)?.id else null

    val current = album
    when {
        loading -> Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            CircularProgressIndicator()
        }
        error != null -> Box(Modifier.fillMaxSize().padding(24.dp), contentAlignment = Alignment.Center) {
            Text(error!!, color = MaterialTheme.colorScheme.error)
        }
        current != null -> AlbumContent(
            album = current,
            liked = liked.value,
            isPlayingId = playingId,
            contentPadding = contentPadding,
            onPlay = { idx -> current.tracks?.let { player.play(it, idx) } },
            onShuffle = {
                current.tracks?.shuffled(Random)?.let { player.play(it, 0) }
            },
            onOpenArtist = onOpenArtist,
            onToggleLike = { track ->
                liked.value.let { current2 ->
                    liked.value = if (track.id in current2) current2 - track.id else current2 + track.id
                }
            },
        )
    }
}

@Composable
private fun AlbumContent(
    album: Album,
    liked: Set<Int>,
    isPlayingId: Int?,
    contentPadding: PaddingValues = PaddingValues(0.dp),
    onPlay: (Int) -> Unit,
    onShuffle: () -> Unit,
    onOpenArtist: (Int) -> Unit,
    onToggleLike: (Track) -> Unit,
) {
    val tracks = album.tracks ?: emptyList()
    val orientation = LocalConfiguration.current.orientation
    val isLandscape = orientation == android.content.res.Configuration.ORIENTATION_LANDSCAPE
    val firstTrackFocus = remember { FocusRequester() }

    LaunchedEffect(Unit) {
        if (tracks.isNotEmpty()) firstTrackFocus.requestFocus()
    }

    val albumMeta = @Composable {
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp)
                .padding(top = 16.dp, bottom = 8.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            ArtworkImage(
                url = album.primaryArtUrl,
                size = if (isLandscape) 200 else 260,
                cornerRadius = 16,
                modifier = Modifier.shadow(
                    24.dp,
                    shape = RoundedCornerShape(16.dp),
                    ambientColor = androidx.compose.ui.graphics.Color.Black.copy(alpha = 0.4f),
                    spotColor = androidx.compose.ui.graphics.Color.Black.copy(alpha = 0.4f),
                ),
            )
            Spacer(Modifier.height(16.dp))
            Text(
                album.name,
                style = MaterialTheme.typography.headlineSmall,
                fontWeight = FontWeight.Bold,
                color = MaterialTheme.colorScheme.onSurface,
            )
            album.artist?.let { a ->
                TextButton(onClick = { onOpenArtist(a.id) }) {
                    Text(a.name, color = MaterialTheme.colorScheme.onSurfaceVariant)
                }
            }
            val meta = buildList {
                album.genres.firstOrNull()?.let { add(it) }
                album.year?.let { add(it.toString()) }
            }.joinToString(" · ")
            if (meta.isNotBlank()) {
                Text(
                    meta,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
            Spacer(Modifier.height(16.dp))
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                FocusableItem(onClick = onShuffle) {
                    FilledTonalButton(onClick = onShuffle) {
                        Icon(Icons.Rounded.Shuffle, contentDescription = "Shuffle")
                        Spacer(Modifier.size(8.dp))
                        Text("Shuffle")
                    }
                }
                FocusableItem(onClick = { onPlay(0) }) {
                    Button(onClick = { onPlay(0) }) {
                        Icon(Icons.Rounded.PlayArrow, contentDescription = "Play")
                        Spacer(Modifier.size(8.dp))
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
                albumMeta()
            }
            LazyColumn(
                Modifier.weight(1f).fillMaxSize(),
                contentPadding = PaddingValues(vertical = 8.dp),
            ) {
                itemsIndexed(tracks, key = { _, t -> t.id }) { index, track ->
                    TrackRow(
                        track = track,
                        trackNumber = track.number ?: index + 1,
                        isLiked = track.id in liked,
                        albumArtistName = album.artist?.name,
                        isPlaying = isPlayingId == track.id,
                        focusRequester = if (index == 0) firstTrackFocus else remember { FocusRequester() },
                        onLike = { onToggleLike(track) },
                        onClick = { onPlay(index) },
                    )
                }
            }
        }
    } else {
        LazyColumn(Modifier.fillMaxSize(), contentPadding = contentPadding) {
            item { albumMeta() }
            itemsIndexed(tracks, key = { _, t -> t.id }) { index, track ->
                TrackRow(
                    track = track,
                    trackNumber = track.number ?: index + 1,
                    isLiked = track.id in liked,
                    albumArtistName = album.artist?.name,
                    isPlaying = isPlayingId == track.id,
                    focusRequester = if (index == 0) firstTrackFocus else remember { FocusRequester() },
                    onLike = { onToggleLike(track) },
                    onClick = { onPlay(index) },
                )
            }
        }
    }
}
