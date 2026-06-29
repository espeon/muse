package vg.nat.muse.ui.player

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.Crossfade
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInHorizontally
import androidx.compose.animation.slideOutHorizontally
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.focusable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsFocusedAsState
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.ExpandMore
import androidx.compose.material.icons.rounded.Favorite
import androidx.compose.material.icons.rounded.FavoriteBorder
import androidx.compose.material.icons.rounded.Pause
import androidx.compose.material.icons.rounded.PlayArrow
import androidx.compose.material.icons.rounded.QueueMusic
import androidx.compose.material.icons.rounded.SkipNext
import androidx.compose.material.icons.rounded.SkipPrevious
import androidx.compose.material.icons.rounded.Subtitles
import androidx.compose.material.icons.rounded.Translate
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.darkColorScheme
import ir.mahozad.multiplatform.wavyslider.material3.WavySlider
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import vg.nat.muse.lyrics.Jlf
import vg.nat.muse.lyrics.LyricsHeader
import vg.nat.muse.lyrics.LyricsScreen
import vg.nat.muse.lyrics.TranslationService
import vg.nat.muse.ui.LocalApiClient
import vg.nat.muse.ui.LocalPlayerEngine
import vg.nat.muse.ui.LocalTranslationService
import vg.nat.muse.ui.LocalUmiClient
import vg.nat.muse.ui.components.ArtworkImage
import vg.nat.muse.ui.components.DynamicArtworkBackground
import vg.nat.muse.ui.components.MarqueeText

@Composable
fun FullPlayer(
    onDismiss: () -> Unit,
    onOpenQueue: () -> Unit,
) {
    val player = LocalPlayerEngine.current
    val api = LocalApiClient.current
    val umi = LocalUmiClient.current
    val translationService = LocalTranslationService.current
    val scope = rememberCoroutineScope()

    val queue by player.queue.collectAsState()
    val index by player.currentIndex.collectAsState()
    val isPlaying by player.isPlaying.collectAsState()
    val position by player.positionSec.collectAsState()
    val duration by player.durationSec.collectAsState()
    val profile by player.currentHlsProfile.collectAsState()
    val track = queue.getOrNull(index)

    var liked by remember(track?.id) { mutableStateOf(track?.liked ?: false) }
    var dragging by remember { mutableStateOf(false) }
    var dragValue by remember { mutableFloatStateOf(0f) }
    var showLyrics by remember { mutableStateOf(false) }
    var lyrics by remember { mutableStateOf<Jlf?>(null) }
    var lyricsLoading by remember { mutableStateOf(false) }
    var lyricsNotFound by remember { mutableStateOf(false) }
    var translatedLines by remember { mutableStateOf<List<String>?>(null) }
    var translating by remember { mutableStateOf(false) }
    var showLanguagePicker by remember { mutableStateOf(false) }

    LaunchedEffect(track?.id) {
        lyrics = null
        lyricsNotFound = false
        translatedLines = null
        val t = track ?: return@LaunchedEffect
        lyricsLoading = true
        val fetched = umi.fetchLyrics(t.name, t.displayArtist, t.albumName)
        lyrics = fetched
        lyricsLoading = false
        lyricsNotFound = fetched == null
    }

    val orientation = LocalConfiguration.current.orientation
    val isLandscape = orientation == android.content.res.Configuration.ORIENTATION_LANDSCAPE
    val playButtonFocus = remember { FocusRequester() }

    LaunchedEffect(Unit) {
        playButtonFocus.requestFocus()
    }

    MaterialTheme(colorScheme = darkColorScheme()) {
        Box(
            Modifier
                .fillMaxSize()
                .onKeyEvent { event ->
                    if (event.type == KeyEventType.KeyDown && event.key == Key(android.view.KeyEvent.KEYCODE_BUTTON_Y)) {
                        showLyrics = !showLyrics
                        true
                    } else false
                }
                .background(
                    Brush.verticalGradient(
                        listOf(
                            MaterialTheme.colorScheme.surfaceVariant,
                            MaterialTheme.colorScheme.surface,
                        ),
                    ),
                ),
        ) {
            DynamicArtworkBackground(
                artworkUrl = track?.artUrl,
                modifier = Modifier.fillMaxSize(),
            )

            if (isLandscape && track != null) {
                val showLyricsView = showLyrics
                val animSpec = tween<Float>(500, easing = FastOutSlowInEasing)
                val artWeight = 1.2f
                val ctrWeight = 1f
                val controlsOffset by animateFloatAsState(
                    targetValue = if (showLyricsView) -(artWeight / ctrWeight) * 1.06f else 0f,
                    animationSpec = animSpec,
                    label = "controlsSlide",
                )
                val artOffset by animateFloatAsState(
                    targetValue = if (showLyricsView) (ctrWeight / artWeight) * 1.06f else 0f,
                    animationSpec = animSpec,
                    label = "artSlide",
                )
                Row(
                    Modifier
                        .fillMaxSize()
                        .windowInsetsPadding(WindowInsets.systemBars)
                        .padding(24.dp),
                    horizontalArrangement = Arrangement.spacedBy(32.dp),
                ) {
                    Box(
                        Modifier
                            .weight(artWeight)
                            .fillMaxSize()
                            .graphicsLayer { translationX = artOffset * size.width },
                    ) {
                        IconButton(
                            onClick = onDismiss,
                            modifier = Modifier.align(Alignment.TopStart),
                        ) {
                            Icon(
                                Icons.Rounded.ExpandMore,
                                contentDescription = "Close",
                                tint = MaterialTheme.colorScheme.onSurface
                            )
                        }
                        Crossfade(
                            targetState = showLyricsView,
                            animationSpec = tween(500),
                            modifier = Modifier.fillMaxSize(),
                        ) { showingLyrics ->
                            if (showingLyrics) {
                                val currentLyrics = lyrics
                                when {
                                    currentLyrics != null -> LyricsScreen(
                                        jlf = currentLyrics,
                                        positionMsProvider = { player.currentPositionMs() },
                                        isPlaying = isPlaying,
                                        onSeek = { player.seekTo(it / 1000.0) },
                                        modifier = Modifier.fillMaxSize(),
                                        translatedLines = translatedLines,
                                    )

                                    lyricsLoading -> Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                        CircularProgressIndicator(color = MaterialTheme.colorScheme.onSurface)
                                    }

                                    lyricsNotFound -> Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                        Text(
                                            "No lyrics found",
                                            color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.6f),
                                            style = MaterialTheme.typography.bodyLarge
                                        )
                                    }
                                }
                            } else {
                                BoxWithConstraints(Modifier.fillMaxSize()) {
                                    val artSize =
                                        (minOf(maxWidth.value, maxHeight.value) - 48f).toInt().coerceAtLeast(160)
                                    Crossfade(
                                        targetState = track.artUrl,
                                        animationSpec = tween(400),
                                        modifier = Modifier.align(Alignment.Center),
                                    ) { url ->
                                        ArtworkImage(
                                            url = url,
                                            size = artSize,
                                            cornerRadius = 20,
                                            modifier = Modifier.clip(RoundedCornerShape(20.dp)),
                                        )
                                    }
                                }
                            }
                        }
                    }
                    Column(
                        Modifier
                            .weight(ctrWeight)
                            .fillMaxSize()
                            .graphicsLayer { translationX = controlsOffset * size.width }
                            .verticalScroll(rememberScrollState()),
                        verticalArrangement = Arrangement.Center,
                        horizontalAlignment = Alignment.CenterHorizontally,
                    ) {
                        ControlsPanel(
                            track = track,
                            liked = liked,
                            isPlaying = isPlaying,
                            position = position,
                            duration = duration,
                            dragging = dragging,
                            dragValue = dragValue,
                            profile = profile,
                            showLyrics = showLyrics,
                            hasLyrics = lyrics != null,
                            hasTranslation = translatedLines != null,
                            artworkUrl = if (showLyricsView) track.artUrl else null,
                            playButtonFocus = playButtonFocus,
                            onToggleLike = {
                                val newLiked = !liked
                                liked = newLiked
                                scope.launch {
                                    try {
                                        val result = withContext(Dispatchers.IO) { api.toggleLike(track.id) }
                                        liked = result.liked
                                    } catch (_: Exception) {
                                        liked = track.liked ?: false
                                    }
                                }
                            },
                            onSeekStart = { dragging = true; dragValue = it },
                            onSeek = { dragValue = it },
                            onSeekEnd = { player.seekTo(dragValue.toDouble()); dragging = false },
                            onPrevious = { player.previous() },
                            onTogglePlay = { player.togglePlayPause() },
                            onNext = { player.next() },
                            onToggleLyrics = { showLyrics = !showLyrics },
                            onTranslate = {
                                if (translatedLines != null) translatedLines = null
                                else showLanguagePicker = true
                            },
                            onOpenQueue = onOpenQueue,
                        )
                    }
                }
            } else {
                Column(
                    Modifier
                        .fillMaxSize()
                        .windowInsetsPadding(WindowInsets.systemBars)
                        .padding(start = 24.dp, end = 24.dp, top = 6.dp, bottom = 36.dp),
                    horizontalAlignment = Alignment.CenterHorizontally,
                ) {
                    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.Center) {
                        IconButton(onClick = onDismiss) {
                            Icon(
                                Icons.Rounded.ExpandMore,
                                contentDescription = "Close",
                                tint = MaterialTheme.colorScheme.onSurface
                            )
                        }
                    }

                    val showLyricsView = showLyrics
                    BoxWithConstraints(Modifier.weight(1f).fillMaxWidth()) {
                        val artSize = (minOf(maxWidth.value, maxHeight.value) - 24f).toInt().coerceAtLeast(160)
                        when {
                            showLyricsView && track != null -> {
                                val currentLyrics = lyrics
                                when {
                                    currentLyrics != null -> LyricsScreen(
                                        jlf = currentLyrics,
                                        positionMsProvider = { player.currentPositionMs() },
                                        isPlaying = isPlaying,
                                        onSeek = { player.seekTo(it / 1000.0) },
                                        modifier = Modifier.fillMaxSize(),
                                        translatedLines = translatedLines,
                                        header = LyricsHeader(
                                            title = track.name,
                                            artist = track.displayArtist,
                                            album = track.albumName,
                                            artUrl = track.artUrl,
                                        ),
                                    )

                                    lyricsLoading -> Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                        CircularProgressIndicator(color = MaterialTheme.colorScheme.onSurface)
                                    }

                                    lyricsNotFound -> Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                        Text(
                                            "No lyrics found",
                                            color = MaterialTheme.colorScheme.onSurface.copy(alpha = 0.6f),
                                            style = MaterialTheme.typography.bodyLarge
                                        )
                                    }

                                    else -> Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                                        CircularProgressIndicator(color = MaterialTheme.colorScheme.onSurface)
                                    }
                                }
                            }

                            track != null -> Crossfade(
                                targetState = track.artUrl,
                                animationSpec = tween(400),
                                modifier = Modifier.align(Alignment.Center),
                            ) { url ->
                                ArtworkImage(
                                    url = url,
                                    size = artSize,
                                    cornerRadius = 20,
                                    modifier = Modifier.clip(RoundedCornerShape(20.dp)),
                                )
                            }
                        }
                    }

                    if (track != null) {
                        ControlsPanel(
                            track = track,
                            liked = liked,
                            isPlaying = isPlaying,
                            position = position,
                            duration = duration,
                            dragging = dragging,
                            dragValue = dragValue,
                            profile = profile,
                            showLyrics = showLyrics,
                            hasLyrics = lyrics != null,
                            hasTranslation = translatedLines != null,
                            playButtonFocus = playButtonFocus,
                            onToggleLike = {
                                val newLiked = !liked
                                liked = newLiked
                                scope.launch {
                                    try {
                                        val result = withContext(Dispatchers.IO) { api.toggleLike(track.id) }
                                        liked = result.liked
                                    } catch (_: Exception) {
                                        liked = track.liked ?: false
                                    }
                                }
                            },
                            onSeekStart = { dragging = true; dragValue = it },
                            onSeek = { dragValue = it },
                            onSeekEnd = { player.seekTo(dragValue.toDouble()); dragging = false },
                            onPrevious = { player.previous() },
                            onTogglePlay = { player.togglePlayPause() },
                            onNext = { player.next() },
                            onToggleLyrics = { showLyrics = !showLyrics },
                            onTranslate = {
                                if (translatedLines != null) translatedLines = null
                                else showLanguagePicker = true
                            },
                            onOpenQueue = onOpenQueue,
                        )
                    }
                }
            }
        }

        if (showLanguagePicker) {
            TranslationLanguagePicker(
                onDismiss = { showLanguagePicker = false },
                onSelect = { language ->
                    showLanguagePicker = false
                    val currentLyrics = lyrics ?: return@TranslationLanguagePicker
                    scope.launch {
                        translating = true
                        try {
                            val result = withContext(Dispatchers.IO) {
                                translationService.translate(currentLyrics, language)
                            }
                            translatedLines = result.lines
                        } catch (_: Exception) {
                        }
                        translating = false
                    }
                },
                currentLanguage = translationService.deviceLanguageCode(),
                showFreeText = translationService.backend == vg.nat.muse.lyrics.TranslationBackend.LLM,
            )
        }

        if (translating) {
            Box(
                Modifier.fillMaxSize().background(Color.Black.copy(alpha = 0.3f)),
                contentAlignment = Alignment.Center,
            ) {
                CircularProgressIndicator(color = MaterialTheme.colorScheme.onSurface)
            }
        }
    }
}

@Composable
private fun ControlsPanel(
    track: vg.nat.muse.net.Track,
    liked: Boolean,
    isPlaying: Boolean,
    position: Double,
    duration: Double,
    dragging: Boolean,
    dragValue: Float,
    profile: vg.nat.muse.net.HlsProfile?,
    showLyrics: Boolean,
    hasLyrics: Boolean,
    hasTranslation: Boolean = false,
    artworkUrl: String? = null,
    playButtonFocus: FocusRequester = remember { FocusRequester() },
    onToggleLike: () -> Unit,
    onSeekStart: (Float) -> Unit,
    onSeek: (Float) -> Unit,
    onSeekEnd: () -> Unit,
    onPrevious: () -> Unit,
    onTogglePlay: () -> Unit,
    onNext: () -> Unit,
    onToggleLyrics: () -> Unit,
    onTranslate: () -> Unit,
    onOpenQueue: () -> Unit,
) {
    Row(
        Modifier.fillMaxWidth(),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        AnimatedVisibility(
            visible = artworkUrl != null,
            enter = slideInHorizontally(tween(300)) { -it } + fadeIn(tween(300)),
            exit = slideOutHorizontally(tween(300)) { -it } + fadeOut(tween(300)),
        ) {
            if (artworkUrl != null) {
                ArtworkImage(
                    url = artworkUrl,
                    size = 52,
                    cornerRadius = 8,
                )
            }
        }
        Column(Modifier.weight(1f)) {
            MarqueeText(
                text = track.name,
                style = MaterialTheme.typography.titleLarge.copy(fontWeight = FontWeight.Bold),
                color = MaterialTheme.colorScheme.onSurface,
            )
            MarqueeText(
                text = track.displayArtist,
                style = MaterialTheme.typography.titleMedium,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        IconButton(onClick = onToggleLike) {
            Icon(
                if (liked) Icons.Rounded.Favorite else Icons.Rounded.FavoriteBorder,
                contentDescription = "Like",
                tint = if (liked) Color(0xFFE91E63) else MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
    }
    Spacer(Modifier.height(20.dp))

    val pos = if (dragging) dragValue.toDouble() else position
    val dur = maxOf(duration, 1.0).toFloat()
    WavySlider(
        value = pos.toFloat().coerceIn(0f, dur),
        onValueChange = { onSeek(it) },
        onValueChangeFinished = onSeekEnd,
        valueRange = 0f..dur,
        modifier = Modifier.fillMaxWidth(),
    )
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
        Text(formatTime(pos), style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.onSurface)
        profile?.displayName?.let {
            Text(it, style = MaterialTheme.typography.bodySmall, color = MaterialTheme.colorScheme.primary)
        }
        Text(
            formatTime(duration),
            style = MaterialTheme.typography.bodySmall,
            color = MaterialTheme.colorScheme.onSurface
        )
    }

    Spacer(Modifier.height(8.dp))
    Row(
        Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceEvenly,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        FocusableButton(onClick = onPrevious) {
            Icon(
                Icons.Rounded.SkipPrevious,
                contentDescription = "Previous",
                tint = MaterialTheme.colorScheme.primary,
                modifier = Modifier.size(40.dp)
            )
        }
        FocusableButton(onClick = onTogglePlay, modifier = Modifier.size(72.dp), focusRequester = playButtonFocus) {
            Icon(
                if (isPlaying) Icons.Rounded.Pause else Icons.Rounded.PlayArrow,
                contentDescription = "Play/Pause",
                tint = MaterialTheme.colorScheme.primary,
                modifier = Modifier.size(64.dp),
            )
        }
        FocusableButton(onClick = onNext) {
            Icon(
                Icons.Rounded.SkipNext,
                contentDescription = "Next",
                tint = MaterialTheme.colorScheme.primary,
                modifier = Modifier.size(40.dp)
            )
        }
    }
    Spacer(Modifier.height(8.dp))
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
        FocusableButton(onClick = onToggleLyrics) {
            Icon(
                Icons.Rounded.Subtitles,
                contentDescription = "Lyrics",
                tint = if (showLyrics) MaterialTheme.colorScheme.primary
                else MaterialTheme.colorScheme.onSurfaceVariant,
            )
        }
        if (hasLyrics) {
            FocusableButton(onClick = onTranslate) {
                Icon(
                    Icons.Rounded.Translate,
                    contentDescription = "Translate",
                    tint = if (hasTranslation) MaterialTheme.colorScheme.primary
                    else MaterialTheme.colorScheme.onSurfaceVariant,
                )
            }
        }
        FocusableButton(onClick = onOpenQueue) {
            Icon(Icons.Rounded.QueueMusic, contentDescription = "Queue", tint = MaterialTheme.colorScheme.onSurface)
        }
    }
}

@Composable
private fun FocusableButton(
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    focusRequester: FocusRequester = remember { FocusRequester() },
    content: @Composable () -> Unit,
) {
    val interactionSource = remember { MutableInteractionSource() }
    val isFocused by interactionSource.collectIsFocusedAsState()
    val scale by animateFloatAsState(
        targetValue = if (isFocused) 1.1f else 1f,
        animationSpec = tween(150),
        label = "btnFocusScale",
    )
    Box(
        modifier = modifier
            .scale(scale)
            .focusRequester(focusRequester)
            .onFocusChanged { }
            .then(
                if (isFocused) Modifier.border(2.dp, MaterialTheme.colorScheme.primary, CircleShape)
                else Modifier,
            )
            .onKeyEvent { event ->
                if (event.type == KeyEventType.KeyDown &&
                    (event.key == Key(ButtonA) || event.key == Key(ButtonDpadCenter))
                ) {
                    onClick()
                    true
                } else false
            }
            .focusable(interactionSource = interactionSource),
        contentAlignment = Alignment.Center,
    ) { content() }
}

private val ButtonA = android.view.KeyEvent.KEYCODE_BUTTON_A
private val ButtonDpadCenter = android.view.KeyEvent.KEYCODE_DPAD_CENTER

private fun formatTime(seconds: Double): String {
    val s = maxOf(0, seconds.toInt())
    return "%d:%02d".format(s / 60, s % 60)
}

private data class LanguageOption(val code: String, val displayName: String)

private val translationLanguages = listOf(
    LanguageOption("af", "Afrikaans"),
    LanguageOption("ar", "Arabic"),
    LanguageOption("be", "Belarusian"),
    LanguageOption("bg", "Bulgarian"),
    LanguageOption("bn", "Bengali"),
    LanguageOption("ca", "Catalan"),
    LanguageOption("cs", "Czech"),
    LanguageOption("cy", "Welsh"),
    LanguageOption("da", "Danish"),
    LanguageOption("de", "German"),
    LanguageOption("el", "Greek"),
    LanguageOption("en", "English"),
    LanguageOption("eo", "Esperanto"),
    LanguageOption("es", "Spanish"),
    LanguageOption("et", "Estonian"),
    LanguageOption("fa", "Persian"),
    LanguageOption("fi", "Finnish"),
    LanguageOption("fr", "French"),
    LanguageOption("ga", "Irish"),
    LanguageOption("gl", "Galician"),
    LanguageOption("gu", "Gujarati"),
    LanguageOption("he", "Hebrew"),
    LanguageOption("hi", "Hindi"),
    LanguageOption("hr", "Croatian"),
    LanguageOption("ht", "Haitian"),
    LanguageOption("hu", "Hungarian"),
    LanguageOption("id", "Indonesian"),
    LanguageOption("is", "Icelandic"),
    LanguageOption("it", "Italian"),
    LanguageOption("ja", "Japanese"),
    LanguageOption("ka", "Georgian"),
    LanguageOption("kn", "Kannada"),
    LanguageOption("ko", "Korean"),
    LanguageOption("lt", "Lithuanian"),
    LanguageOption("lv", "Latvian"),
    LanguageOption("mk", "Macedonian"),
    LanguageOption("mr", "Marathi"),
    LanguageOption("ms", "Malay"),
    LanguageOption("mt", "Maltese"),
    LanguageOption("nl", "Dutch"),
    LanguageOption("no", "Norwegian"),
    LanguageOption("pl", "Polish"),
    LanguageOption("pt", "Portuguese"),
    LanguageOption("ro", "Romanian"),
    LanguageOption("ru", "Russian"),
    LanguageOption("sk", "Slovak"),
    LanguageOption("sl", "Slovenian"),
    LanguageOption("sq", "Albanian"),
    LanguageOption("sv", "Swedish"),
    LanguageOption("sw", "Swahili"),
    LanguageOption("ta", "Tamil"),
    LanguageOption("te", "Telugu"),
    LanguageOption("th", "Thai"),
    LanguageOption("tl", "Tagalog"),
    LanguageOption("tr", "Turkish"),
    LanguageOption("uk", "Ukrainian"),
    LanguageOption("ur", "Urdu"),
    LanguageOption("vi", "Vietnamese"),
    LanguageOption("zh", "Chinese"),
)

@OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)
@Composable
private fun TranslationLanguagePicker(
    onDismiss: () -> Unit,
    onSelect: (String) -> Unit,
    currentLanguage: String,
    showFreeText: Boolean = false,
) {
    val context = LocalContext.current
    val prefs = remember { context.getSharedPreferences("translation_prefs", 0) }
    var recentCodes by remember {
        mutableStateOf(prefs.getStringSet("recent_languages", emptySet())?.toList()?.take(6) ?: emptyList())
    }
    val recentLanguages = remember(recentCodes) {
        recentCodes.mapNotNull { code -> translationLanguages.find { it.code == code } }
    }
    val sheetState = androidx.compose.material3.rememberModalBottomSheetState(
        skipPartiallyExpanded = true,
    )

    fun saveRecent(code: String) {
        val updated = (listOf(code) + recentCodes).distinct().take(6)
        recentCodes = updated
        prefs.edit().putStringSet("recent_languages", updated.toSet()).apply()
    }

    androidx.compose.material3.ModalBottomSheet(
        onDismissRequest = onDismiss,
        sheetState = sheetState,
    ) {
        Column(Modifier.padding(bottom = 32.dp)) {
            Text(
                "Translate Lyrics",
                style = MaterialTheme.typography.titleLarge,
                fontWeight = FontWeight.SemiBold,
                color = MaterialTheme.colorScheme.onSurface,
                modifier = Modifier.padding(horizontal = 20.dp, vertical = 12.dp),
            )
            if (showFreeText) {
                var customLanguage by remember { mutableStateOf("") }
                Row(
                    Modifier.fillMaxWidth().padding(horizontal = 20.dp, vertical = 4.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    OutlinedTextField(
                        value = customLanguage,
                        onValueChange = { customLanguage = it },
                        placeholder = { Text("Any language...") },
                        singleLine = true,
                        modifier = Modifier.weight(1f),
                    )
                    androidx.compose.material3.FilledTonalButton(
                        onClick = {
                            if (customLanguage.isNotBlank()) {
                                onSelect(customLanguage.trim())
                            }
                        },
                        enabled = customLanguage.isNotBlank(),
                    ) {
                        Text("Go")
                    }
                }
                Spacer(Modifier.height(8.dp))
            }
            androidx.compose.foundation.lazy.LazyColumn {
                if (recentLanguages.isNotEmpty()) {
                    item {
                        Text(
                            "Recent",
                            style = MaterialTheme.typography.labelLarge,
                            color = MaterialTheme.colorScheme.primary,
                            modifier = Modifier.padding(horizontal = 20.dp, vertical = 8.dp),
                        )
                    }
                    items(recentLanguages) { lang ->
                        LanguageRow(
                            lang = lang,
                            isDeviceLanguage = lang.code == currentLanguage,
                            onClick = {
                                saveRecent(lang.code)
                                onSelect(lang.code)
                            },
                        )
                    }
                    item {
                        Spacer(Modifier.height(8.dp))
                        HorizontalDivider(Modifier.padding(horizontal = 20.dp))
                        Spacer(Modifier.height(8.dp))
                    }
                }
                item {
                    Text(
                        "All Languages",
                        style = MaterialTheme.typography.labelLarge,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                        modifier = Modifier.padding(horizontal = 20.dp, vertical = 8.dp),
                    )
                }
                items(translationLanguages) { lang ->
                    LanguageRow(
                        lang = lang,
                        isDeviceLanguage = lang.code == currentLanguage,
                        onClick = {
                            saveRecent(lang.code)
                            onSelect(lang.code)
                        },
                    )
                }
            }
        }
    }
}

@Composable
private fun LanguageRow(
    lang: LanguageOption,
    isDeviceLanguage: Boolean,
    onClick: () -> Unit,
) {
    Row(
        Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
            .padding(horizontal = 20.dp, vertical = 14.dp),
        horizontalArrangement = Arrangement.SpaceBetween,
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Text(lang.displayName, color = MaterialTheme.colorScheme.onSurface)
        if (isDeviceLanguage) {
            Text(
                "Device",
                style = MaterialTheme.typography.bodySmall,
                color = MaterialTheme.colorScheme.primary,
            )
        }
    }
}
