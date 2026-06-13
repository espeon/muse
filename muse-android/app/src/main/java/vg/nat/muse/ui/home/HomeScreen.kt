package vg.nat.muse.ui.home

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyRow
import androidx.compose.foundation.lazy.items
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
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import vg.nat.muse.net.ApiException
import vg.nat.muse.net.HomeRow
import vg.nat.muse.ui.LocalApiClient
import vg.nat.muse.ui.components.AlbumCard
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext

@Composable
fun HomeScreen(
    onOpenAlbum: (Int) -> Unit,
    contentPadding: PaddingValues = PaddingValues(0.dp),
) {
    val api = LocalApiClient.current
    var rows by remember { mutableStateOf<List<HomeRow>>(emptyList()) }
    var loading by remember { mutableStateOf(true) }
    var error by remember { mutableStateOf<String?>(null) }

    LaunchedEffect(Unit) {
        loading = true
        error = null
        error = try {
            rows = withContext(Dispatchers.IO) { api.fetchHome() }
            null
        } catch (e: ApiException) {
            when (e) {
                is ApiException.Unauthorized -> "Unauthorized. Please log in again."
                is ApiException.NotFound -> "Not found."
                is ApiException.ServerError -> "Server error."
                is ApiException.Network -> "Network error: ${e.message}"
            }
        } catch (e: Exception) {
            e.message
        }
        loading = false
    }

    when {
        loading -> Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            CircularProgressIndicator()
        }
        error != null -> Box(Modifier.fillMaxSize().padding(24.dp), contentAlignment = Alignment.Center) {
            Text(error!!, color = MaterialTheme.colorScheme.onSurfaceVariant)
        }
        else -> LazyColumn(
            modifier = Modifier.fillMaxSize(),
            contentPadding = contentPadding,
        ) {
            items(rows, key = { it.name }) { row ->
                HomeRowSection(row = row, onOpenAlbum = onOpenAlbum)
                Spacer(Modifier.height(28.dp))
            }
        }
    }
}

@Composable
private fun HomeRowSection(row: HomeRow, onOpenAlbum: (Int) -> Unit) {
    Spacer(Modifier.height(4.dp))
    Text(
        text = row.name,
        style = MaterialTheme.typography.titleLarge,
        fontWeight = FontWeight.Bold,
        modifier = Modifier.padding(horizontal = 16.dp),
    )
    Spacer(Modifier.height(12.dp))
    LazyRow(
        contentPadding = PaddingValues(horizontal = 16.dp),
        horizontalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        items(row.albums, key = { it.id }) { album ->
            AlbumCard(album = album, onClick = { onOpenAlbum(album.id) })
        }
    }
}
