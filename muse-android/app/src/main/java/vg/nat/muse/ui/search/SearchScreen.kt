package vg.nat.muse.ui.search

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.clickable
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
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Close
import androidx.compose.material.icons.rounded.History
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FilterChip
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
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
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.async
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.delay
import vg.nat.muse.net.AlbumPartial
import vg.nat.muse.net.ArtistPartial
import vg.nat.muse.net.SearchResult
import vg.nat.muse.ui.FocusableItem
import vg.nat.muse.ui.LocalApiClient
import vg.nat.muse.ui.LocalPlayerEngine
import vg.nat.muse.ui.components.AlbumCard
import vg.nat.muse.ui.components.ArtworkImage
import vg.nat.muse.ui.toTrack

private const val RECENT_KEY = "recent_searches"
private const val MAX_RECENT = 10

private enum class SongSort(val label: String, val sortby: String, val dir: String) {
    Default("Default", "id", "asc"),
    SongAZ("Song A-Z", "song", "asc"),
    SongZA("Song Z-A", "song", "desc"),
    ArtistAZ("Artist A-Z", "artist", "asc"),
    AlbumAZ("Album A-Z", "album", "asc"),
}

private data class SearchResults(
    val songs: List<SearchResult>,
    val albums: List<AlbumPartial>,
    val artists: List<ArtistPartial>,
)

@Composable
fun SearchScreen(
    contentPadding: PaddingValues = PaddingValues(0.dp),
    onOpenAlbum: (Int) -> Unit = {},
    onOpenArtist: (Int) -> Unit = {},
) {
    val api = LocalApiClient.current
    val player = LocalPlayerEngine.current
    val context = LocalContext.current
    val prefs = remember { context.getSharedPreferences("search_prefs", 0) }

    var query by remember { mutableStateOf("") }
    var results by remember { mutableStateOf<SearchResults?>(null) }
    var searching by remember { mutableStateOf(false) }
    var sort by remember { mutableStateOf(SongSort.Default) }
    var recentSearches by remember {
        mutableStateOf(prefs.getStringSet(RECENT_KEY, emptySet())?.toList()?.take(MAX_RECENT) ?: emptyList())
    }

    fun saveRecent(q: String) {
        val updated = (listOf(q) + recentSearches).distinct().take(MAX_RECENT)
        recentSearches = updated
        prefs.edit().putStringSet(RECENT_KEY, updated.toSet()).apply()
    }

    fun removeRecent(q: String) {
        val updated = recentSearches.filter { it != q }
        recentSearches = updated
        prefs.edit().putStringSet(RECENT_KEY, updated.toSet()).apply()
    }

    LaunchedEffect(query, sort) {
        if (query.isBlank()) {
            results = null
            searching = false
            return@LaunchedEffect
        }
        delay(350)
        searching = true
        try {
            val searchResults = coroutineScope {
                val songs = async(Dispatchers.IO) {
                    api.searchSongs(
                        query,
                        sortby = sort.sortby.takeIf { sort != SongSort.Default },
                        dir = sort.dir.takeIf { sort != SongSort.Default },
                    )
                }
                val albums = async(Dispatchers.IO) {
                    api.fetchAlbums(cursor = 0, limit = 5, filter = query)
                }
                val artists = async(Dispatchers.IO) {
                    api.fetchArtists(cursor = 0, limit = 5, filter = query)
                }
                SearchResults(
                    songs = songs.await(),
                    albums = albums.await().albums,
                    artists = artists.await().artists,
                )
            }
            results = searchResults
        } catch (_: Exception) {
            results = null
        }
        searching = false
    }

    Column(Modifier.fillMaxSize().padding(contentPadding)) {
        OutlinedTextField(
            value = query,
            onValueChange = { query = it },
            placeholder = { Text("Search songs, albums, artists") },
            leadingIcon = { Icon(Icons.Rounded.Search, contentDescription = null) },
            trailingIcon = {
                AnimatedVisibility(visible = query.isNotEmpty(), enter = fadeIn(), exit = fadeOut()) {
                    IconButton(onClick = {
                        query = ""
                        results = null
                    }) {
                        Icon(Icons.Rounded.Close, contentDescription = "Clear")
                    }
                }
            },
            singleLine = true,
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp, vertical = 8.dp),
        )

        when {
            query.isBlank() -> {
                if (recentSearches.isNotEmpty()) {
                    RecentSearchesList(
                        searches = recentSearches,
                        onSelect = { query = it },
                        onRemove = { removeRecent(it) },
                    )
                } else {
                    Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                        Text("Search your library", color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                }
            }
            searching -> Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                CircularProgressIndicator()
            }
            results == null -> Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                Text("Search failed", color = MaterialTheme.colorScheme.onSurfaceVariant)
            }
            else -> {
                val r = results!!
                if (r.songs.isEmpty() && r.albums.isEmpty() && r.artists.isEmpty()) {
                    Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                        Text("No results", color = MaterialTheme.colorScheme.onSurfaceVariant)
                    }
                } else {
                    LaunchedEffect(query) { saveRecent(query) }

                    LazyColumn(
                        Modifier.fillMaxSize(),
                        contentPadding = PaddingValues(bottom = 16.dp),
                    ) {
                        if (r.artists.isNotEmpty()) {
                            item(key = "artists_header") {
                                SectionHeader("Artists")
                            }
                            items(r.artists, key = { "artist_${it.id}" }) { artist ->
                                ArtistRow(
                                    artist = artist,
                                    onClick = { onOpenArtist(artist.id) },
                                )
                            }
                        }

                        if (r.albums.isNotEmpty()) {
                            item(key = "albums_header") {
                                SectionHeader("Albums")
                            }
                            item(key = "albums_row") {
                                LazyRow(
                                    contentPadding = PaddingValues(horizontal = 16.dp),
                                    horizontalArrangement = Arrangement.spacedBy(12.dp),
                                ) {
                                    items(r.albums, key = { "album_${it.id}" }) { album ->
                                        AlbumCard(
                                            album = album,
                                            onClick = { onOpenAlbum(album.id) },
                                        )
                                    }
                                }
                                Spacer(Modifier.height(8.dp))
                            }
                        }

                        if (r.songs.isNotEmpty()) {
                            item(key = "songs_header") {
                                Row(
                                    Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 8.dp),
                                    horizontalArrangement = Arrangement.SpaceBetween,
                                    verticalAlignment = Alignment.CenterVertically,
                                ) {
                                    Text(
                                        "Songs",
                                        style = MaterialTheme.typography.titleMedium,
                                        fontWeight = FontWeight.SemiBold,
                                        color = MaterialTheme.colorScheme.onSurface,
                                    )
                                    SortChip(sort = sort, onSelect = { sort = it })
                                }
                            }
                            items(r.songs, key = { "song_${it.id}" }) { result ->
                                FocusableItem(
                                    onClick = { player.play(listOf(result.toTrack()), 0) },
                                    modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 4.dp),
                                ) {
                                    Row(
                                        verticalAlignment = Alignment.CenterVertically,
                                        horizontalArrangement = Arrangement.spacedBy(12.dp),
                                    ) {
                                        ArtworkImage(url = result.picture, size = 44, cornerRadius = 6)
                                        Column(Modifier.weight(1f)) {
                                            Text(
                                                result.songName,
                                                style = MaterialTheme.typography.bodyLarge,
                                                color = MaterialTheme.colorScheme.onSurface,
                                                maxLines = 1,
                                                overflow = TextOverflow.Ellipsis,
                                            )
                                            val subtitle = listOfNotNull(result.artistName, result.albumName)
                                                .joinToString(" - ")
                                            if (subtitle.isNotEmpty()) {
                                                Text(
                                                    subtitle,
                                                    style = MaterialTheme.typography.bodySmall,
                                                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                                                    maxLines = 1,
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
    }
}

@Composable
private fun SectionHeader(title: String) {
    Text(
        text = title,
        style = MaterialTheme.typography.titleMedium,
        fontWeight = FontWeight.SemiBold,
        color = MaterialTheme.colorScheme.onSurface,
        modifier = Modifier.padding(start = 16.dp, top = 16.dp, bottom = 8.dp),
    )
}

@Composable
private fun ArtistRow(artist: ArtistPartial, onClick: () -> Unit) {
    FocusableItem(
        onClick = onClick,
        modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 4.dp),
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            ArtworkImage(url = artist.picture, size = 44, cornerRadius = 22)
            Column(Modifier.weight(1f)) {
                Text(
                    artist.name,
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.onSurface,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                artist.numAlbums?.let { count ->
                    Text(
                        "$count album${if (count != 1) "s" else ""}",
                        style = MaterialTheme.typography.bodySmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                }
            }
        }
    }
}

@Composable
private fun SortChip(sort: SongSort, onSelect: (SongSort) -> Unit) {
    var expanded by remember { mutableStateOf(false) }

    Box {
        FilterChip(
            selected = sort != SongSort.Default,
            onClick = { expanded = true },
            label = { Text(sort.label, style = MaterialTheme.typography.bodySmall) },
        )
        if (expanded) {
            androidx.compose.material3.DropdownMenu(
                expanded = expanded,
                onDismissRequest = { expanded = false },
            ) {
                SongSort.entries.forEach { option ->
                    androidx.compose.material3.DropdownMenuItem(
                        text = { Text(option.label) },
                        onClick = {
                            onSelect(option)
                            expanded = false
                        },
                    )
                }
            }
        }
    }
}

@Composable
private fun RecentSearchesList(
    searches: List<String>,
    onSelect: (String) -> Unit,
    onRemove: (String) -> Unit,
) {
    Column(Modifier.padding(horizontal = 16.dp)) {
        Text(
            "Recent Searches",
            style = MaterialTheme.typography.titleMedium,
            fontWeight = FontWeight.SemiBold,
            color = MaterialTheme.colorScheme.onSurface,
            modifier = Modifier.padding(vertical = 12.dp),
        )
        searches.forEach { search ->
            Row(
                Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(8.dp))
                    .clickable { onSelect(search) }
                    .padding(vertical = 10.dp, horizontal = 4.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Icon(
                    Icons.Rounded.History,
                    contentDescription = null,
                    tint = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.size(20.dp),
                )
                Spacer(Modifier.width(12.dp))
                Text(
                    search,
                    style = MaterialTheme.typography.bodyLarge,
                    color = MaterialTheme.colorScheme.onSurface,
                    modifier = Modifier.weight(1f),
                )
                IconButton(onClick = { onRemove(search) }, modifier = Modifier.size(24.dp)) {
                    Icon(
                        Icons.Rounded.Close,
                        contentDescription = "Remove",
                        tint = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.size(16.dp),
                    )
                }
            }
        }
    }
}
