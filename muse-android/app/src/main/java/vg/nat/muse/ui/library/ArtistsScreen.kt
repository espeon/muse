package vg.nat.muse.ui.library

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import vg.nat.muse.net.ArtistPartial
import vg.nat.muse.ui.LocalApiClient
import vg.nat.muse.ui.components.ArtworkImage

private const val PAGE_SIZE = 50

@Composable
fun ArtistsScreen(onOpenArtist: (Int) -> Unit) {
    val api = LocalApiClient.current
    var artists by remember { mutableStateOf<List<ArtistPartial>>(emptyList()) }
    var cursor by remember { mutableStateOf(0) }
    var hasMore by remember { mutableStateOf(true) }
    var loading by remember { mutableStateOf(false) }
    var filter by remember { mutableStateOf("") }
    var lastFilter by remember { mutableStateOf("") }

    suspend fun loadMore() {
        if (loading || !hasMore) return
        loading = true
        try {
            val result = withContext(Dispatchers.IO) {
                api.fetchArtists(cursor = cursor, limit = PAGE_SIZE, filter = filter.ifBlank { null })
            }
            artists = artists + result.artists
            cursor = result.cursor + result.artists.size
            hasMore = result.artists.size == PAGE_SIZE
        } catch (_: Exception) {
            hasMore = false
        }
        loading = false
    }

    LaunchedEffect(filter) {
        if (filter == lastFilter) return@LaunchedEffect
        delay(300)
        cursor = 0
        hasMore = true
        artists = emptyList()
        lastFilter = filter
        loadMore()
    }

    Column(Modifier.fillMaxSize()) {
        OutlinedTextField(
            value = filter,
            onValueChange = { filter = it },
            placeholder = { Text("Search artists") },
            singleLine = true,
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp, vertical = 8.dp),
        )
        LazyColumn(Modifier.fillMaxSize()) {
            items(artists, key = { it.id }) { artist ->
                ArtistRow(artist = artist, onClick = { onOpenArtist(artist.id) })
                LaunchedEffect(artist.id) {
                    if (artist.id == artists.lastOrNull()?.id && hasMore && !loading) loadMore()
                }
            }
            if (loading) {
                item {
                    Box(
                        Modifier.fillMaxWidth().padding(16.dp),
                        contentAlignment = Alignment.Center,
                    ) { CircularProgressIndicator() }
                }
            }
        }
    }
}

@Composable
private fun ArtistRow(artist: ArtistPartial, onClick: () -> Unit) {
    Row(
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(12.dp),
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
            .padding(horizontal = 16.dp, vertical = 6.dp),
    ) {
        ArtworkImage(url = artist.picture, size = 44, cornerRadius = 22)
        Column {
            Text(
                text = artist.name,
                style = MaterialTheme.typography.bodyLarge,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            artist.numAlbums?.let { count ->
                Text(
                    text = "$count ${if (count == 1) "album" else "albums"}",
                    style = MaterialTheme.typography.bodySmall,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
    }
}
