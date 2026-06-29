package vg.nat.muse.player

import android.content.Context
import android.net.Uri
import androidx.media3.common.AudioAttributes
import androidx.media3.common.C
import androidx.media3.common.MediaItem
import androidx.media3.common.MediaMetadata
import androidx.media3.common.MimeTypes
import androidx.media3.common.PlaybackException
import androidx.media3.common.Player
import androidx.media3.common.TrackSelectionParameters
import androidx.media3.exoplayer.ExoPlayer
import androidx.media3.exoplayer.analytics.AnalyticsListener
import androidx.media3.exoplayer.source.MediaLoadData
import kotlinx.coroutines.Job
import kotlinx.coroutines.MainScope
import kotlinx.coroutines.cancel
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import vg.nat.muse.net.ApiClient
import vg.nat.muse.net.HlsProfile
import vg.nat.muse.net.Track
import kotlin.math.abs

class PlayerEngine(
    private val appContext: Context,
    private val apiClient: ApiClient,
) {
    private val scope = MainScope()
    private val prefs = appContext.getSharedPreferences(PREFS, Context.MODE_PRIVATE)

    val player: ExoPlayer = ExoPlayer.Builder(appContext)
        .setAudioAttributes(
            AudioAttributes.Builder()
                .setContentType(C.AUDIO_CONTENT_TYPE_MUSIC)
                .setUsage(C.USAGE_MEDIA)
                .build(),
            /* handleAudioFocus = */ true,
        )
        .setHandleAudioBecomingNoisy(true)
        .build()
        .also {
            it.addListener(PlayerListener())
            it.addAnalyticsListener(QualityListener())
        }

    private val _queue = MutableStateFlow<List<Track>>(emptyList())
    val queue: StateFlow<List<Track>> = _queue.asStateFlow()

    private val _currentIndex = MutableStateFlow(0)
    val currentIndex: StateFlow<Int> = _currentIndex.asStateFlow()

    private val _isPlaying = MutableStateFlow(false)
    val isPlaying: StateFlow<Boolean> = _isPlaying.asStateFlow()

    private val _positionSec = MutableStateFlow(0.0)
    val positionSec: StateFlow<Double> = _positionSec.asStateFlow()

    private val _durationSec = MutableStateFlow(0.0)
    val durationSec: StateFlow<Double> = _durationSec.asStateFlow()

    private val _volume = MutableStateFlow(1f)
    val volume: StateFlow<Float> = _volume.asStateFlow()

    private val _hlsProfiles = MutableStateFlow<List<HlsProfile>>(emptyList())
    val hlsProfiles: StateFlow<List<HlsProfile>> = _hlsProfiles.asStateFlow()

    private val _currentHlsProfile = MutableStateFlow<HlsProfile?>(null)
    val currentHlsProfile: StateFlow<HlsProfile?> = _currentHlsProfile.asStateFlow()

    var useHls: Boolean
        get() = prefs.getBoolean(KEY_USE_HLS, false)
        set(value) {
            prefs.edit().putBoolean(KEY_USE_HLS, value).apply()
        }

    var selectedProfile: String?
        get() = prefs.getString(KEY_HLS_PROFILE, null)
        set(value) {
            prefs.edit().putString(KEY_HLS_PROFILE, value).apply()
        }

    val currentTrack: Track?
        get() = _queue.value.getOrNull(_currentIndex.value)

    fun currentPositionMs(): Long = player.currentPosition

    private var baseIndex = 0
    private val signedUrls = mutableMapOf<Int, String>()
    private var lastDetectedProfile: HlsProfile? = null
    private var tickerJob: Job? = null

    fun play(tracks: List<Track>, startIndex: Int = 0) {
        scope.launch { playImpl(tracks, startIndex) }
    }

    private suspend fun playImpl(tracks: List<Track>, startIndex: Int) {
        if (tracks.isEmpty()) return
        _queue.value = tracks
        _currentIndex.value = startIndex
        baseIndex = startIndex
        signedUrls.clear()

        if (useHls && _hlsProfiles.value.isEmpty()) {
            _hlsProfiles.value = runCatching { apiClient.fetchHlsProfiles() }.getOrDefault(emptyList())
        }

        val toSign = (startIndex until minOf(startIndex + PRELOAD_COUNT + 1, tracks.size)).toList()
        if (!signInto(toSign, tracks)) return

        val items = toSign.mapNotNull { mediaItemFor(tracks[it]) }
        player.setMediaItems(items, /* startWindowIndex = */ 0, /* startPositionMs = */ 0L)
        applyQualityParameters()
        player.prepare()
        player.play()
        scope.launch { runCatching { apiClient.setPlaying(tracks[startIndex].id) } }
    }

    private suspend fun enqueueUpcoming() {
        val q = _queue.value
        if (q.isEmpty()) return
        val nextStart = baseIndex + player.mediaItemCount
        val end = minOf(nextStart + PRELOAD_COUNT, q.size)
        if (nextStart >= end) return
        val toSign = (nextStart until end).filter { signedUrls[q[it].id] == null }
        if (!signInto(toSign, q)) return
        val items = (nextStart until end).mapNotNull { mediaItemFor(q[it]) }
        if (items.isNotEmpty()) player.addMediaItems(items)
    }

    private suspend fun signInto(indices: List<Int>, tracks: List<Track>): Boolean {
        if (indices.isEmpty()) return true
        val ids = indices.map { tracks[it].id }
        return try {
            apiClient.batchSignTracks(ids = ids, mode = if (useHls) "hls" else null).forEach {
                signedUrls[it.id] = it.url
            }
            true
        } catch (e: Exception) {
            false
        }
    }

    private fun mediaItemFor(track: Track): MediaItem? {
        val url = signedUrls[track.id] ?: return null
        val builder = MediaItem.Builder()
            .setMediaId(track.id.toString())
            .setUri(url)
            .setMediaMetadata(
                MediaMetadata.Builder()
                    .setTitle(track.name)
                    .setArtist(track.displayArtist)
                    .setAlbumTitle(track.albumName)
                    .apply { track.artUrl?.let { setArtworkUri(Uri.parse(it)) } }
                    .build(),
            )
        if (useHls) builder.setMimeType(MimeTypes.APPLICATION_M3U8)
        return builder.build()
    }

    fun togglePlayPause() {
        if (player.isPlaying) player.pause() else player.play()
    }

    fun next() {
        if (player.hasNextMediaItem()) player.seekToNextMediaItem()
    }

    fun previous() {
        if (_positionSec.value > 3.0) {
            player.seekTo(0L)
            return
        }
        if (player.hasPreviousMediaItem()) player.seekToPreviousMediaItem() else player.seekTo(0L)
    }

    fun seekTo(positionSeconds: Double) {
        player.seekTo((positionSeconds * 1000).toLong().coerceAtLeast(0L))
        _positionSec.value = positionSeconds
    }

    fun setVolume(value: Float) {
        val v = value.coerceIn(0f, 1f)
        player.volume = v
        _volume.value = v
    }

    fun setQuality(profileName: String?) {
        selectedProfile = profileName
        applyQualityParameters()
    }

    suspend fun loadHlsProfiles() {
        if (_hlsProfiles.value.isNotEmpty()) return
        _hlsProfiles.value = runCatching { apiClient.fetchHlsProfiles() }.getOrDefault(emptyList())
    }

    private fun applyQualityParameters() {
        if (!useHls) return
        val maxBitrate = maxAudioBitrateFor(selectedProfile)
        player.trackSelectionParameters =
            player.trackSelectionParameters
                .buildUpon()
                .setMaxAudioBitrate(maxBitrate)
                .build()
    }

    private fun maxAudioBitrateFor(profileName: String?): Int {
        if (profileName == null) return 0
        val profile = _hlsProfiles.value.firstOrNull { it.name == profileName } ?: return 0
        return profile.bitrate ?: 0
    }

    private fun updateCurrentQuality(indicatedBitrate: Long) {
        val profiles = _hlsProfiles.value
        if (profiles.isEmpty() || indicatedBitrate <= 0) {
            _currentHlsProfile.value = null
            lastDetectedProfile = null
            return
        }
        val maxLossy = profiles.mapNotNull { it.bitrate }.maxOrNull() ?: 0
        val detected: HlsProfile? = if (indicatedBitrate > maxLossy + LOSSLESS_THRESHOLD) {
            profiles.firstOrNull { it.bitrate == null }
        } else {
            profiles.minByOrNull { p ->
                val target = (p.bitrate ?: Int.MAX_VALUE).toLong()
                abs(target - indicatedBitrate)
            }
        }
        if (detected?.name == lastDetectedProfile?.name) {
            _currentHlsProfile.value = detected
        }
        lastDetectedProfile = detected
    }

    private fun startTicker() {
        if (tickerJob?.isActive == true) return
        tickerJob = scope.launch {
            while (isActive) {
                delay(TICKER_INTERVAL_MS)
                val pos = player.currentPosition
                if (pos >= 0) _positionSec.value = pos / 1000.0
                val dur = player.duration
                if (dur != C.TIME_UNSET && dur > 0) _durationSec.value = dur / 1000.0
            }
        }
    }

    private fun stopTicker() {
        tickerJob?.cancel()
        tickerJob = null
    }

    private inner class PlayerListener : Player.Listener {
        override fun onIsPlayingChanged(isPlaying: Boolean) {
            _isPlaying.value = isPlaying
            if (isPlaying) startTicker() else stopTicker()
        }

        override fun onMediaItemTransition(mediaItem: MediaItem?, reason: Int) {
            val logical = baseIndex + player.currentMediaItemIndex
            val q = _queue.value
            if (logical in q.indices) {
                _currentIndex.value = logical
                _currentHlsProfile.value = null
                lastDetectedProfile = null
                val track = q[logical]
                scope.launch {
                    enqueueUpcoming()
                    runCatching { apiClient.setPlaying(track.id) }
                }
            }
            val dur = player.duration
            if (dur != C.TIME_UNSET && dur > 0) _durationSec.value = dur / 1000.0
        }

        override fun onPlaybackStateChanged(state: Int) {
            if (state == Player.STATE_ENDED) stopTicker()
            val dur = player.duration
            if (dur != C.TIME_UNSET && dur > 0) _durationSec.value = dur / 1000.0
        }

        override fun onPositionDiscontinuity(
            oldPosition: Player.PositionInfo,
            newPosition: Player.PositionInfo,
            reason: Int,
        ) {
            val logical = baseIndex + player.currentMediaItemIndex
            if (logical in _queue.value.indices && logical != _currentIndex.value) {
                _currentIndex.value = logical
            }
            _positionSec.value = player.currentPosition.coerceAtLeast(0) / 1000.0
        }

        override fun onPlayerError(error: PlaybackException) {
            stopTicker()
        }
    }

    private inner class QualityListener : AnalyticsListener {
        override fun onDownstreamFormatChanged(
            eventTime: AnalyticsListener.EventTime,
            mediaLoadData: MediaLoadData,
        ) {
            if (!useHls) return
            if (mediaLoadData.trackType != C.TRACK_TYPE_AUDIO) return
            val bitrate = mediaLoadData.trackFormat?.bitrate ?: return
            if (bitrate > 0) updateCurrentQuality(bitrate.toLong())
        }
    }

    fun release() {
        stopTicker()
        scope.cancel()
        player.release()
    }

    private companion object {
        const val PREFS = "muse_player"
        const val KEY_USE_HLS = "use_hls"
        const val KEY_HLS_PROFILE = "hls_profile"
        const val PRELOAD_COUNT = 3
        const val LOSSLESS_THRESHOLD = 560_000L
        const val TICKER_INTERVAL_MS = 500L
    }
}
