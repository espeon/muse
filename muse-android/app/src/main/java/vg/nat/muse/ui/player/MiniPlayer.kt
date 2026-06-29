package vg.nat.muse.ui.player

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.focusable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsFocusedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Pause
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.SkipNext
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import vg.nat.muse.ui.LocalPlayerEngine
import vg.nat.muse.ui.components.ArtworkImage

private val MiniPlayerShape = RoundedCornerShape(16.dp)

@Composable
fun MiniPlayer(onExpand: () -> Unit) {
    val player = LocalPlayerEngine.current
    val queue by player.queue.collectAsState()
    val index by player.currentIndex.collectAsState()
    val isPlaying by player.isPlaying.collectAsState()
    val track = queue.getOrNull(index) ?: return

    val interactionSource = remember { MutableInteractionSource() }
    val isFocused by interactionSource.collectIsFocusedAsState()

    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        modifier = Modifier
            .fillMaxWidth()
            .height(64.dp)
            .then(
                if (isFocused) Modifier.border(2.dp, MaterialTheme.colorScheme.primary, MiniPlayerShape)
                else Modifier,
            )
            .clip(MiniPlayerShape)
            .background(MaterialTheme.colorScheme.surfaceVariant)
            .onPreviewKeyEvent { event ->
                if (event.type == KeyEventType.KeyDown &&
                    (event.key == Key(ButtonA) || event.key == Key(ButtonDpadCenter))
                ) {
                    onExpand()
                    true
                } else false
            }
            .focusable(interactionSource = interactionSource)
            .padding(horizontal = 12.dp),
    ) {
        ArtworkImage(url = track.artUrl, size = 44, cornerRadius = 6)
        Column(Modifier.weight(1f)) {
            Text(
                track.name,
                style = MaterialTheme.typography.bodyMedium,
                color = MaterialTheme.colorScheme.onSurface,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Text(
                track.displayArtist,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
        IconButton(onClick = { player.togglePlayPause() }) {
            Icon(
                if (isPlaying) Icons.Rounded.Pause else Icons.Rounded.PlayArrow,
                contentDescription = "Play/Pause",
            )
        }
        IconButton(onClick = { player.next() }) {
            Icon(Icons.Rounded.SkipNext, contentDescription = "Next")
        }
    }
}

private val ButtonA = android.view.KeyEvent.KEYCODE_BUTTON_A
private val ButtonDpadCenter = android.view.KeyEvent.KEYCODE_DPAD_CENTER
