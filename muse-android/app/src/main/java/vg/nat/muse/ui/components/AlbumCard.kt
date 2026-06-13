package vg.nat.muse.ui.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import vg.nat.muse.net.AlbumPartial

@Composable
fun AlbumCard(
    album: AlbumPartial,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    width: Int = 160,
) {
    Column(
        horizontalAlignment = Alignment.Start,
        modifier = modifier.width(width.dp).clickable(onClick = onClick),
    ) {
        ArtworkImage(
            url = album.primaryArtUrl,
            size = width,
            cornerRadius = 10,
            modifier = Modifier.shadow(
                elevation = 6.dp,
                shape = RoundedCornerShape(10.dp),
                ambientColor = Color0_15,
                spotColor = Color0_15,
            ),
        )
        Spacer(Modifier.height(6.dp))
        Text(
            text = album.name,
            style = MaterialTheme.typography.labelLarge,
            color = MaterialTheme.colorScheme.onSurface,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        val subtitle = album.artist?.name ?: album.year?.toString()
        if (subtitle != null) {
            Text(
                text = subtitle,
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

private val Color0_15 = androidx.compose.ui.graphics.Color.Black.copy(alpha = 0.15f)
