package vg.nat.muse.ui.library

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun LibraryScreen(
    onOpenAlbum: (Int) -> Unit,
    onOpenArtist: (Int) -> Unit,
    onOpenPlaylist: (Int) -> Unit,
    contentPadding: PaddingValues = PaddingValues(0.dp),
) {
    var selected by remember { mutableIntStateOf(0) }
    val options = listOf("Albums", "Artists", "Playlists")

    Column(Modifier.fillMaxSize().padding(contentPadding)) {
        SingleChoiceSegmentedButtonRow(
            modifier = Modifier
                .fillMaxSize()
                .padding(horizontal = 16.dp, vertical = 8.dp),
        ) {
            options.forEachIndexed { index, label ->
                SegmentedButton(
                    selected = selected == index,
                    onClick = { selected = index },
                    shape = SegmentedButtonDefaults.itemShape(index, options.size),
                ) { Text(label) }
            }
        }
        when (selected) {
            0 -> AlbumsScreen(onOpenAlbum = onOpenAlbum)
            1 -> ArtistsScreen(onOpenArtist = onOpenArtist)
            2 -> PlaylistsScreen(onOpenPlaylist = onOpenPlaylist)
        }
    }
}
