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
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.CircularProgressIndicator
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
import androidx.compose.ui.draw.clip
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import vg.nat.muse.net.Artist
import vg.nat.muse.ui.FocusableItem
import vg.nat.muse.ui.LocalApiClient
import vg.nat.muse.ui.components.ArtworkImage

@Composable
fun ArtistDetailScreen(
    artistId: Int,
    onOpenAlbum: (Int) -> Unit,
    contentPadding: PaddingValues = PaddingValues(0.dp),
) {
    val api = LocalApiClient.current
    var artist by remember { mutableStateOf<Artist?>(null) }
    var loading by remember { mutableStateOf(true) }
    var error by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(artistId) {
        loading = true
        error = null
        try {
            artist = withContext(Dispatchers.IO) { api.fetchArtist(artistId) }
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
        artist != null -> {
            val a = artist!!
            val orientation = LocalConfiguration.current.orientation
            val isLandscape = orientation == android.content.res.Configuration.ORIENTATION_LANDSCAPE
            val firstAlbumFocus = remember { FocusRequester() }

            LaunchedEffect(Unit) {
                if (a.albums.isNotEmpty()) firstAlbumFocus.requestFocus()
            }

            val artistMeta = @Composable {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .padding(top = 24.dp, bottom = 16.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    ArtworkImage(
                        url = a.picture,
                        size = if (isLandscape) 120 else 150,
                        modifier = Modifier.clip(CircleShape),
                    )
                    Spacer(Modifier.height(16.dp))
                    Text(a.name, style = MaterialTheme.typography.headlineMedium, color = MaterialTheme.colorScheme.onSurface)
                    a.bio?.takeIf { it.isNotBlank() }?.let {
                        Spacer(Modifier.height(8.dp))
                        Text(
                            it,
                            style = MaterialTheme.typography.bodyMedium,
                            color = MaterialTheme.colorScheme.onSurfaceVariant,
                            modifier = Modifier.padding(horizontal = 24.dp),
                        )
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
                        artistMeta()
                    }
                    LazyColumn(
                        Modifier.weight(1f).fillMaxSize(),
                        contentPadding = PaddingValues(vertical = 8.dp),
                    ) {
                        itemsIndexed(a.albums, key = { _, album -> album.id }) { index, album ->
                            FocusableItem(
                                onClick = { onOpenAlbum(album.id) },
                                modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 6.dp),
                                focusRequester = if (index == 0) firstAlbumFocus else remember { FocusRequester() },
                            ) {
                                Row(
                                    verticalAlignment = Alignment.CenterVertically,
                                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                                ) {
                                    ArtworkImage(url = album.primaryArtUrl, size = 52, cornerRadius = 6)
                                    Column {
                                        Text(
                                            album.name,
                                            style = MaterialTheme.typography.bodyLarge,
                                            color = MaterialTheme.colorScheme.onSurface,
                                            maxLines = 1,
                                            overflow = TextOverflow.Ellipsis,
                                        )
                                        album.year?.let {
                                            Text(
                                                it.toString(),
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
            } else {
                LazyColumn(Modifier.fillMaxSize(), contentPadding = contentPadding) {
                    item { artistMeta() }
                    itemsIndexed(a.albums, key = { _, album -> album.id }) { index, album ->
                        FocusableItem(
                            onClick = { onOpenAlbum(album.id) },
                            modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 6.dp),
                            focusRequester = if (index == 0) firstAlbumFocus else remember { FocusRequester() },
                        ) {
                            Row(
                                verticalAlignment = Alignment.CenterVertically,
                                horizontalArrangement = Arrangement.spacedBy(12.dp),
                            ) {
                                ArtworkImage(url = album.primaryArtUrl, size = 52, cornerRadius = 6)
                                Column {
                                    Text(
                                        album.name,
                                        style = MaterialTheme.typography.bodyLarge,
                                        color = MaterialTheme.colorScheme.onSurface,
                                        maxLines = 1,
                                        overflow = TextOverflow.Ellipsis,
                                    )
                                    album.year?.let {
                                        Text(
                                            it.toString(),
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
    }
}
