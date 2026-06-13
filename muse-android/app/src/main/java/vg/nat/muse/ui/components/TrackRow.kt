package vg.nat.muse.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Favorite
import androidx.compose.material.icons.filled.FavoriteBorder
import androidx.compose.material.icons.rounded.Equalizer
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import vg.nat.muse.net.Track

@Composable
fun TrackRow(
    track: Track,
    modifier: Modifier = Modifier,
    trackNumber: Int? = null,
    isLiked: Boolean = false,
    albumArtistName: String? = null,
    isPlaying: Boolean = false,
    onLike: (() -> Unit)? = null,
    onClick: (() -> Unit)? = null,
) {
    val showArtist = albumArtistName?.let { track.displayArtist != it } ?: true
    Row(
        verticalAlignment = Alignment.CenterVertically,
        modifier = modifier
            .fillMaxWidth()
            .clickable(enabled = onClick != null) { onClick?.invoke() }
            .padding(horizontal = 16.dp, vertical = 10.dp),
    ) {
        val accent = MaterialTheme.colorScheme.primary
        when {
            isPlaying -> Icon(
                imageVector = Icons.Rounded.Equalizer,
                contentDescription = null,
                tint = accent,
                modifier = Modifier.width(24.dp),
            )
            trackNumber != null -> Text(
                text = trackNumber.toString(),
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.width(24.dp),
            )
            else -> Spacer(Modifier.width(24.dp))
        }

        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = track.name,
                style = MaterialTheme.typography.bodyLarge,
                color = if (isPlaying) accent else MaterialTheme.colorScheme.onSurface,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            if (showArtist) {
                Text(
                    text = track.displayArtist,
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
            }
        }

        if (track.lossless == true) {
            LosslessBadge(Modifier.padding(end = 8.dp))
        }

        Text(
            text = track.formattedDuration,
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )

        if (onLike != null) {
            Icon(
                imageVector = if (isLiked) Icons.Filled.Favorite else Icons.Filled.FavoriteBorder,
                contentDescription = null,
                tint = if (isLiked) androidx.compose.ui.graphics.Color(0xFFE91E63)
                else MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier
                    .padding(start = 12.dp)
                    .clip(RoundedCornerShape(50))
                    .clickable(onClick = onLike)
                    .padding(4.dp),
            )
        }
    }
}

@Composable
fun LosslessBadge(modifier: Modifier = Modifier) {
    Text(
        text = "HI-RES",
        fontSize = 9.sp,
        fontWeight = FontWeight.Bold,
        color = androidx.compose.ui.graphics.Color.White,
        modifier = modifier
            .clip(RoundedCornerShape(6.dp))
            .background(MaterialTheme.colorScheme.primary.copy(alpha = 0.25f))
            .padding(horizontal = 5.dp, vertical = 2.dp),
    )
}
