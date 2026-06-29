package vg.nat.muse.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.MusicNote
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import coil3.compose.AsyncImage

@Composable
fun ArtworkImage(
    url: String?,
    size: Int,
    modifier: Modifier = Modifier,
    cornerRadius: Int = 8,
) {
    val shape = RoundedCornerShape(cornerRadius.dp)
    Box(
        modifier = modifier
            .size(size.dp)
            .clip(shape)
            .background(MaterialTheme.colorScheme.surfaceVariant)
            .border(1.dp, Color.Black.copy(alpha = 0.1f), shape),
    ) {
        Icon(
            imageVector = Icons.Rounded.MusicNote,
            contentDescription = null,
            tint = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.5f),
            modifier = Modifier
                .align(Alignment.Center)
                .size((size * 0.3f).dp),
        )
        if (!url.isNullOrEmpty()) {
            AsyncImage(
                model = url,
                contentDescription = null,
                contentScale = ContentScale.Crop,
                modifier = Modifier.fillMaxSize(),
            )
        }
    }
}
