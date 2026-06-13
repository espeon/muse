package vg.nat.muse.ui.player

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.ExpandMore
import androidx.compose.material.icons.rounded.Favorite
import androidx.compose.material.icons.rounded.FavoriteBorder
import androidx.compose.material.icons.rounded.Pause
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.QueueMusic
import androidx.compose.material.icons.rounded.SkipNext
import androidx.compose.material.icons.rounded.SkipPrevious
import androidx.compose.material.icons.rounded.Subtitles
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import vg.nat.muse.lyrics.Jlf
import vg.nat.muse.lyrics.LyricsScreen
import vg.nat.muse.ui.LocalApiClient
import vg.nat.muse.ui.LocalPlayerEngine
import vg.nat.muse.ui.LocalUmiClient
import vg.nat.muse.ui.components.ArtworkImage
import vg.nat.muse.ui.components.DynamicArtworkBackground

@Composable
fun FullPlayer(
    onDismiss: () -> Unit,
    onOpenQueue: () -> Unit,
) {
    val player = LocalPlayerEngine.current
    val api = LocalApiClient.current
    val umi = LocalUmiClient.current
    val scope = rememberCoroutineScope()

    val queue by player.queue.collectAsState()
    val index by player.currentIndex.collectAsState()
    val isPlaying by player.isPlaying.collectAsState()
    val position by player.positionSec.collectAsState()
    val duration by player.durationSec.collectAsState()
    val profile by player.currentHlsProfile.collectAsState()
    val track = queue.getOrNull(index)

    var liked by remember(track?.id) { mutableStateOf(track?.liked ?: false) }
    var dragging by remember { mutableStateOf(false) }
    var dragValue by remember { mutableFloatStateOf(0f) }
    var showLyrics by remember { mutableStateOf(false) }
    var lyrics by remember { mutableStateOf<Jlf?>(null) }

    LaunchedEffect(track?.id) {
        lyrics = null
        val t = track ?: return@LaunchedEffect
        lyrics = umi.fetchLyrics(t.name, t.displayArtist, t.albumName)
    }

    Box(
        Modifier
            .fillMaxSize()
            .background(
                Brush.verticalGradient(
                    listOf(
                        MaterialTheme.colorScheme.surfaceVariant,
                        MaterialTheme.colorScheme.surface,
                    ),
                ),
            ),
    ) {
        DynamicArtworkBackground(
            artworkUrl = track?.artUrl,
            modifier = Modifier.fillMaxSize(),
        )
        Column(
            Modifier
                .fillMaxSize()
                .windowInsetsPadding(WindowInsets.systemBars)
                .padding(24.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.Center) {
                IconButton(onClick = onDismiss) {
                    Icon(Icons.Rounded.ExpandMore, contentDescription = "Close")
                }
            }
            val showLyricsView = showLyrics && lyrics != null
            if (showLyricsView) {
                LyricsScreen(
                    jlf = lyrics!!,
                    currentTimeMs = (position * 1000).toInt(),
                    onSeek = { player.seekTo(it / 1000.0) },
                    modifier = Modifier.weight(1f).fillMaxWidth(),
                )
            } else {
                Spacer(Modifier.weight(1f))
            }
            if (track != null) {
                if (!showLyricsView) {
                    ArtworkImage(
                        url = track.artUrl,
                        size = 300,
                        cornerRadius = 20,
                        modifier = Modifier.clip(RoundedCornerShape(20.dp)),
                    )
                    Spacer(Modifier.height(32.dp))
                }
                Row(
                    Modifier.fillMaxWidth(),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.SpaceBetween,
                ) {
                    Column(Modifier.weight(1f)) {
                        Text(
                            track.name,
                            style = MaterialTheme.typography.titleLarge,
                            fontWeight = FontWeight.Bold,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                        Text(
                            track.displayArtist,
                            style = MaterialTheme.typography.titleMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                    }
                    IconButton(onClick = {
                        val newLiked = !liked
                        liked = newLiked
                        val id = track.id
                        scope.launch {
                            try {
                                val result = withContext(Dispatchers.IO) { api.toggleLike(id) }
                                liked = result.liked
                            } catch (_: Exception) {
                                liked = track.liked ?: false
                            }
                        }
                    }) {
                        Icon(
                            if (liked) Icons.Rounded.Favorite else Icons.Rounded.FavoriteBorder,
                            contentDescription = "Like",
                            tint = if (liked) Color(0xFFE91E63) else MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                }
                Spacer(Modifier.height(24.dp))

                val pos = if (dragging) dragValue.toDouble() else position
                val dur = maxOf(duration, 1.0).toFloat()
                Slider(
                    value = pos.toFloat().coerceIn(0f, dur),
                    onValueChange = { dragValue = it; dragging = true },
                    onValueChangeFinished = {
                        player.seekTo(dragValue.toDouble())
                        dragging = false
                    },
                    valueRange = 0f..dur,
                )
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                    Text(formatTime(pos), style = MaterialTheme.typography.bodySmall)
                    profile?.displayName?.let {
                        Text(it, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.primary)
                    }
                    Text(formatTime(duration), style = MaterialTheme.typography.bodySmall)
                }

                Spacer(Modifier.height(16.dp))
                Row(
                    Modifier.fillMaxWidth(),
                    horizontalArrangement = Arrangement.SpaceEvenly,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    IconButton(onClick = { player.previous() }) {
                        Icon(Icons.Rounded.SkipPrevious, contentDescription = "Previous", modifier = Modifier.size(40.dp))
                    }
                    IconButton(onClick = { player.togglePlayPause() }, modifier = Modifier.size(72.dp)) {
                        Icon(
                            if (isPlaying) Icons.Rounded.Pause else Icons.Rounded.PlayArrow,
                            contentDescription = "Play/Pause",
                            modifier = Modifier.size(64.dp),
                        )
                    }
                    IconButton(onClick = { player.next() }) {
                        Icon(Icons.Rounded.SkipNext, contentDescription = "Next", modifier = Modifier.size(40.dp))
                    }
                }
                Spacer(Modifier.height(16.dp))
                Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                    IconButton(onClick = { showLyrics = !showLyrics }) {
                        Icon(
                            Icons.Rounded.Subtitles,
                            contentDescription = "Lyrics",
                            tint = if (showLyrics) MaterialTheme.colorScheme.primary
                            else MaterialTheme.colorScheme.onSurfaceVariant,
                        )
                    }
                    IconButton(onClick = onOpenQueue) {
                        Icon(Icons.Rounded.QueueMusic, contentDescription = "Queue")
                    }
                }
            }
            if (!showLyricsView) Spacer(Modifier.weight(1f))
        }
    }
}

private fun formatTime(seconds: Double): String {
    val s = maxOf(0, seconds.toInt())
    return "%d:%02d".format(s / 60, s % 60)
}
