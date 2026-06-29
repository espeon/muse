package vg.nat.muse.ui.library

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.material3.CircularProgressIndicator
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
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import vg.nat.muse.net.AlbumPartial
import vg.nat.muse.ui.LocalApiClient
import vg.nat.muse.ui.components.AlbumCard

private const val PAGE_SIZE = 50

@Composable
fun AlbumsScreen(onOpenAlbum: (Int) -> Unit) {
    val api = LocalApiClient.current
    var albums by remember { mutableStateOf<List<AlbumPartial>>(emptyList()) }
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
                api.fetchAlbums(cursor = cursor, limit = PAGE_SIZE, filter = filter.ifBlank { null })
            }
            albums = albums + result.albums
            cursor = result.offset + result.albums.size
            hasMore = result.albums.size == PAGE_SIZE
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
        albums = emptyList()
        lastFilter = filter
        loadMore()
    }

    androidx.compose.foundation.layout.Column(Modifier.fillMaxSize()) {
        OutlinedTextField(
            value = filter,
            onValueChange = { filter = it },
            placeholder = { Text("Search albums") },
            singleLine = true,
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp, vertical = 8.dp),
        )
        Box(Modifier.fillMaxSize()) {
            LazyVerticalGrid(
                columns = GridCells.Fixed(2),
                contentPadding = androidx.compose.foundation.layout.PaddingValues(16.dp),
                horizontalArrangement = Arrangement.spacedBy(16.dp),
                modifier = Modifier.fillMaxSize(),
            ) {
                items(albums, key = { it.id }) { album ->
                    AlbumCard(album = album, onClick = { onOpenAlbum(album.id) })
                    LaunchedEffect(album.id) {
                        if (album.id == albums.lastOrNull()?.id && hasMore && !loading) loadMore()
                    }
                }
                if (loading) {
                    item(span = { androidx.compose.foundation.lazy.grid.GridItemSpan(2) }) {
                        Box(Modifier.fillMaxWidth().padding(16.dp), contentAlignment = Alignment.Center) {
                            CircularProgressIndicator()
                        }
                    }
                }
            }
        }
    }
}
