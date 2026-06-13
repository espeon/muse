package vg.nat.muse.ui

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Home
import androidx.compose.material.icons.rounded.LibraryMusic
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material.icons.rounded.Settings
import androidx.compose.material3.Icon
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.navigation.NavType
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import androidx.navigation.navArgument
import vg.nat.muse.ui.detail.AlbumDetailScreen
import vg.nat.muse.ui.detail.ArtistDetailScreen
import vg.nat.muse.ui.detail.PlaylistDetailScreen
import vg.nat.muse.ui.home.HomeScreen
import vg.nat.muse.ui.library.LibraryScreen
import vg.nat.muse.ui.player.FullPlayer
import vg.nat.muse.ui.player.MiniPlayer
import vg.nat.muse.ui.player.PlayerSheet
import vg.nat.muse.ui.player.QueueSheet
import vg.nat.muse.ui.search.SearchScreen
import vg.nat.muse.ui.settings.SettingsScreen

private sealed class Tab(val route: String, val label: String, val icon: androidx.compose.ui.graphics.vector.ImageVector) {
    data object Home : Tab("home", "Home", Icons.Rounded.Home)
    data object Library : Tab("library", "Library", Icons.Rounded.LibraryMusic)
    data object Search : Tab("search", "Search", Icons.Rounded.Search)
    data object Settings : Tab("settings", "Settings", Icons.Rounded.Settings)
}

@Composable
fun RootScaffold() {
    val navController = rememberNavController()
    val player = LocalPlayerEngine.current
    val queue by player.queue.collectAsState()
    val index by player.currentIndex.collectAsState()
    val hasTrack = queue.getOrNull(index) != null

    var showFullPlayer by rememberSaveable { mutableStateOf(false) }
    var showQueue by rememberSaveable { mutableStateOf(false) }

    val tabs = listOf(Tab.Home, Tab.Library, Tab.Search, Tab.Settings)
    val backStack by navController.currentBackStackEntryAsState()
    val currentRoute = backStack?.destination?.route

    BackHandler(enabled = showFullPlayer) { showFullPlayer = false }

    Box(Modifier.fillMaxSize()) {
        Scaffold(
            bottomBar = {
                Column {
                    if (hasTrack) {
                        MiniPlayer(
                            onExpand = { showFullPlayer = true },
                        )
                    }
                    NavigationBar {
                        tabs.forEach { tab ->
                            NavigationBarItem(
                                selected = currentRoute?.startsWith(tab.route) == true,
                                onClick = {
                                    navController.navigate(tab.route) {
                                        popUpTo(navController.graph.startDestinationId) { saveState = true }
                                        launchSingleTop = true
                                        restoreState = true
                                    }
                                },
                                icon = { Icon(tab.icon, contentDescription = tab.label) },
                                label = { Text(tab.label) },
                            )
                        }
                    }
                }
            },
        ) { innerPadding ->
            NavHost(
                navController = navController,
                startDestination = Tab.Home.route,
                modifier = Modifier.fillMaxSize(),
            ) {
                composable(Tab.Home.route) {
                    HomeScreen(
                        onOpenAlbum = { navController.navigate("album/$it") },
                        contentPadding = innerPadding,
                    )
                }
                composable(Tab.Library.route) {
                    LibraryScreen(
                        onOpenAlbum = { navController.navigate("album/$it") },
                        onOpenArtist = { navController.navigate("artist/$it") },
                        onOpenPlaylist = { navController.navigate("playlist/$it") },
                    )
                }
                composable(Tab.Search.route) { SearchScreen() }
                composable(Tab.Settings.route) { SettingsScreen() }
                composable(
                    route = "album/{albumId}",
                    arguments = listOf(navArgument("albumId") { type = NavType.IntType }),
                ) { entry ->
                    AlbumDetailScreen(
                        albumId = entry.arguments?.getInt("albumId") ?: 0,
                        onOpenArtist = { navController.navigate("artist/$it") },
                    )
                }
                composable(
                    route = "artist/{artistId}",
                    arguments = listOf(navArgument("artistId") { type = NavType.IntType }),
                ) { entry ->
                    ArtistDetailScreen(
                        artistId = entry.arguments?.getInt("artistId") ?: 0,
                        onOpenAlbum = { navController.navigate("album/$it") },
                    )
                }
                composable(
                    route = "playlist/{playlistId}",
                    arguments = listOf(navArgument("playlistId") { type = NavType.IntType }),
                ) { entry ->
                    PlaylistDetailScreen(playlistId = entry.arguments?.getInt("playlistId") ?: 0)
                }
            }
        }

        if (showFullPlayer) {
            PlayerSheet(
                visible = showFullPlayer,
                onDismiss = { showFullPlayer = false },
            ) {
                FullPlayer(
                    onDismiss = { showFullPlayer = false },
                    onOpenQueue = { showQueue = true },
                )
            }
        }
        if (showQueue) {
            QueueSheet(onDismiss = { showQueue = false })
        }
    }
}
