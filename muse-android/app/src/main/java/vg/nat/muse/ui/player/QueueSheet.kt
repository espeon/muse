package vg.nat.muse.ui.player

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Equalizer
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.Text
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import vg.nat.muse.ui.LocalPlayerEngine
import vg.nat.muse.ui.components.ArtworkImage

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun QueueSheet(onDismiss: () -> Unit) {
    val player = LocalPlayerEngine.current
    val queue by player.queue.collectAsState()
    val index by player.currentIndex.collectAsState()
    val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)

    ModalBottomSheet(onDismissRequest = onDismiss, sheetState = sheetState) {
        Text(
            "Queue",
            style = MaterialTheme.typography.titleMedium,
            modifier = Modifier.padding(start = 16.dp, bottom = 8.dp),
        )
        LazyColumn(Modifier.fillMaxWidth()) {
            itemsIndexed(queue, key = { _, t -> t.id }) { i, track ->
                Row(
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(horizontal = 16.dp, vertical = 4.dp),
                ) {
                    ArtworkImage(url = track.artUrl, size = 44, cornerRadius = 6)
                    Column(Modifier.weight(1f)) {
                        Text(
                            track.name,
                            style = MaterialTheme.typography.bodyMedium,
                            color = if (i == index) MaterialTheme.colorScheme.primary
                            else MaterialTheme.colorScheme.onSurface,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                        )
                        Text(
                            track.displayArtist,
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            maxLines = 1,
                        )
                    }
                    if (i == index) {
                        Icon(Icons.Rounded.Equalizer, contentDescription = null, tint = MaterialTheme.colorScheme.primary)
                    }
                }
            }
        }
        Box(Modifier.size(0.dp).padding(8.dp)) {}
    }
}
