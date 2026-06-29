package vg.nat.muse.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Home
import androidx.compose.material.icons.rounded.LibraryMusic
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material.icons.rounded.Settings
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationRail
import androidx.compose.material3.NavigationRailItem
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.unit.dp
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
    val hasGamepad = LocalHasGamepad.current
    val queue by player.queue.collectAsState()
    val index by player.currentIndex.collectAsState()
    val hasTrack = queue.getOrNull(index) != null

    var showFullPlayer by rememberSaveable { mutableStateOf(false) }
    var showQueue by rememberSaveable { mutableStateOf(false) }

    val tabs = listOf(Tab.Home, Tab.Library, Tab.Search, Tab.Settings)
    val backStack by navController.currentBackStackEntryAsState()
    val currentRoute = backStack?.destination?.route

    val isPlaying by player.isPlaying.collectAsState()

    LaunchedEffect(isPlaying) {
        if (isPlaying && hasTrack && !showFullPlayer) {
            showFullPlayer = true
        }
    }

    val orientation = LocalConfiguration.current.orientation
    val isLandscape = orientation == android.content.res.Configuration.ORIENTATION_LANDSCAPE
    val contentFocus = remember { FocusRequester() }

    LaunchedEffect(currentRoute) {
        runCatching { contentFocus.requestFocus() }
    }

    fun navigateTab(tab: Tab) {
        navController.navigate(tab.route) {
            popUpTo(navController.graph.startDestinationId) { saveState = true }
            launchSingleTop = true
            restoreState = true
        }
    }

    ControllerActions(
        hasGamepad = hasGamepad,
        onDismiss = {
            when {
                showQueue -> showQueue = false
                showFullPlayer -> showFullPlayer = false
                else -> navController.popBackStack()
            }
        },
        onTogglePlayer = { if (hasTrack) showFullPlayer = !showFullPlayer },
        onSwitchTab = { direction ->
            val currentIdx = tabs.indexOfFirst { currentRoute?.startsWith(it.route) == true }
            if (currentIdx >= 0) {
                navigateTab(tabs[(currentIdx + direction + tabs.size) % tabs.size])
            }
        },
    ) {
        Box(Modifier.fillMaxSize()) {
            if (isLandscape) {
                Row(
                    Modifier
                        .fillMaxSize()
                        .windowInsetsPadding(WindowInsets.systemBars),
                ) {
                    Column(
                        Modifier
                            .fillMaxHeight()
                            .background(MaterialTheme.colorScheme.surfaceVariant),
                    ) {
                        NavigationRail {
                            tabs.forEach { tab ->
                                NavigationRailItem(
                                    selected = currentRoute?.startsWith(tab.route) == true,
                                    onClick = { navigateTab(tab) },
                                    icon = { Icon(tab.icon, contentDescription = tab.label) },
                                    label = { Text(tab.label) },
                                )
                            }
                        }
                    }
                    Box(
                        Modifier
                            .fillMaxSize()
                            .focusRequester(contentFocus)
                            .padding(start = 24.dp)
                            .windowInsetsPadding(WindowInsets.navigationBars),
                    ) {
                        val contentPadding = PaddingValues(bottom = if (hasTrack) 80.dp else 0.dp)
                        NavHost(
                            navController = navController,
                            startDestination = Tab.Home.route,
                            modifier = Modifier.fillMaxSize(),
                        ) {
                            composable(Tab.Home.route) {
                                HomeScreen(
                                    onOpenAlbum = { navController.navigate("album/$it") },
                                    contentPadding = contentPadding,
                                )
                            }
                            composable(Tab.Library.route) {
                                LibraryScreen(
                                    onOpenAlbum = { navController.navigate("album/$it") },
                                    onOpenArtist = { navController.navigate("artist/$it") },
                                    onOpenPlaylist = { navController.navigate("playlist/$it") },
                                    contentPadding = contentPadding,
                                )
                            }
                            composable(Tab.Search.route) {
                                SearchScreen(
                                    contentPadding = contentPadding,
                                    onOpenAlbum = { navController.navigate("album/$it") },
                                    onOpenArtist = { navController.navigate("artist/$it") },
                                )
                            }
                            composable(Tab.Settings.route) { SettingsScreen(contentPadding = contentPadding) }
                            composable(
                                route = "album/{albumId}",
                                arguments = listOf(navArgument("albumId") { type = NavType.IntType }),
                            ) { entry ->
                                AlbumDetailScreen(
                                    albumId = entry.arguments?.getInt("albumId") ?: 0,
                                    onOpenArtist = { navController.navigate("artist/$it") },
                                    contentPadding = contentPadding,
                                )
                            }
                            composable(
                                route = "artist/{artistId}",
                                arguments = listOf(navArgument("artistId") { type = NavType.IntType }),
                            ) { entry ->
                                ArtistDetailScreen(
                                    artistId = entry.arguments?.getInt("artistId") ?: 0,
                                    onOpenAlbum = { navController.navigate("album/$it") },
                                    contentPadding = contentPadding,
                                )
                            }
                            composable(
                                route = "playlist/{playlistId}",
                                arguments = listOf(navArgument("playlistId") { type = NavType.IntType }),
                            ) { entry ->
                                PlaylistDetailScreen(
                                    playlistId = entry.arguments?.getInt("playlistId") ?: 0,
                                    contentPadding = contentPadding,
                                )
                            }
                        }
                        if (hasTrack) {
                            Box(
                                Modifier
                                    .align(Alignment.BottomCenter)
                                    .padding(horizontal = 24.dp, vertical = 12.dp)
                                    .widthIn(max = 480.dp)
                                    .fillMaxWidth(),
                            ) {
                                MiniPlayer(onExpand = { showFullPlayer = true })
                            }
                        }
                    }
                }
            } else {
                Scaffold(
                    bottomBar = {
                        Column {
                            if (hasTrack) {
                                MiniPlayer(onExpand = { showFullPlayer = true })
                            }
                            NavigationBar {
                                tabs.forEach { tab ->
                                    NavigationBarItem(
                                        selected = currentRoute?.startsWith(tab.route) == true,
                                        onClick = { navigateTab(tab) },
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
                        modifier = Modifier.fillMaxSize().focusRequester(contentFocus),
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
                                contentPadding = innerPadding,
                            )
                        }
                        composable(Tab.Search.route) {
                            SearchScreen(
                                contentPadding = innerPadding,
                                onOpenAlbum = { navController.navigate("album/$it") },
                                onOpenArtist = { navController.navigate("artist/$it") },
                            )
                        }
                        composable(Tab.Settings.route) { SettingsScreen(contentPadding = innerPadding) }
                        composable(
                            route = "album/{albumId}",
                            arguments = listOf(navArgument("albumId") { type = NavType.IntType }),
                        ) { entry ->
                            AlbumDetailScreen(
                                albumId = entry.arguments?.getInt("albumId") ?: 0,
                                onOpenArtist = { navController.navigate("artist/$it") },
                                contentPadding = innerPadding,
                            )
                        }
                        composable(
                            route = "artist/{artistId}",
                            arguments = listOf(navArgument("artistId") { type = NavType.IntType }),
                        ) { entry ->
                            ArtistDetailScreen(
                                artistId = entry.arguments?.getInt("artistId") ?: 0,
                                onOpenAlbum = { navController.navigate("album/$it") },
                                contentPadding = innerPadding,
                            )
                        }
                        composable(
                            route = "playlist/{playlistId}",
                            arguments = listOf(navArgument("playlistId") { type = NavType.IntType }),
                        ) { entry ->
                            PlaylistDetailScreen(
                                playlistId = entry.arguments?.getInt("playlistId") ?: 0,
                                contentPadding = innerPadding,
                            )
                        }
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
}
