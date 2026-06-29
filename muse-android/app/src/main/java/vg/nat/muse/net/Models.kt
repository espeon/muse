package vg.nat.muse.net

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import java.time.Instant

@Serializable
data class ArtistPartial(
    val id: Int,
    val slug: String? = null,
    val name: String,
    val picture: String? = null,
    val numAlbums: Int? = null,
)

@Serializable
data class Artist(
    val id: Int,
    val slug: String,
    val name: String,
    val picture: String? = null,
    val tags: String? = null,
    val bio: String? = null,
    @Serializable(with = FlexInstantSerializer::class) val createdAt: Instant,
    @Serializable(with = FlexInstantSerializer::class) val updatedAt: Instant? = null,
    val albums: List<AlbumPartial>,
) {
    val tagList: List<String>
        get() = tags
            ?.takeIf { it.isNotBlank() }
            ?.split(",")
            ?.map { it.trim() }
            ?.filter { it.isNotEmpty() }
            ?: emptyList()
}

@Serializable
data class AllArtistsPartial(
    val artists: List<ArtistPartial>,
    val limit: Int,
    val cursor: Int,
)

@Serializable
data class AlbumPartial(
    val id: Int,
    val slug: String? = null,
    val name: String,
    val disambiguation: String? = null,
    val art: List<String>,
    val year: Int? = null,
    val count: Int? = null,
    val artist: ArtistPartial? = null,
) {
    val primaryArtUrl: String? get() = art.firstOrNull()
}

@Serializable
data class Album(
    val id: Int,
    val name: String,
    val art: List<String>,
    val year: Int? = null,
    val genres: List<String>,
    val copyright: String? = null,
    val label: String? = null,
    @Serializable(with = FlexInstantSerializer::class) val createdAt: Instant,
    @Serializable(with = FlexInstantSerializer::class) val updatedAt: Instant? = null,
    val artist: ArtistPartial,
    val tracks: List<Track>? = null,
) {
    val primaryArtUrl: String? get() = art.firstOrNull()
}

@Serializable
data class AllAlbumsPartial(
    val albums: List<AlbumPartial>,
    val limit: Int,
    val offset: Int,
)

@Serializable
data class Track(
    val id: Int,
    val name: String,
    val albumArtist: Int,
    val artists: List<ArtistPartial>,
    val plays: Int? = null,
    val duration: Int,
    val liked: Boolean? = null,
    @Serializable(with = FlexInstantSerializer::class) val lastPlay: Instant? = null,
    val year: Int? = null,
    val number: Int? = null,
    val disc: Int? = null,
    val lossless: Boolean? = null,
    val sampleRate: Int? = null,
    val bitsPerSample: Int? = null,
    val numChannels: Int? = null,
    val composer: String? = null,
    val isrc: String? = null,
    val bpm: Int? = null,
    @Serializable(with = FlexInstantSerializer::class) val createdAt: Instant,
    @Serializable(with = FlexInstantSerializer::class) val updatedAt: Instant? = null,
    val album: Int,
    val albumName: String,
    val artistName: String,
    val artUrl: String? = null,
) {
    val formattedDuration: String
        get() = formatDuration(duration)

    val displayArtist: String get() = artistName
}

@Serializable
data class TrackListItem(
    val id: Int,
    val name: String,
    val duration: Int,
    val number: Int? = null,
    val disc: Int? = null,
    val lossless: Boolean? = null,
    val sampleRate: Int? = null,
    val bitsPerSample: Int? = null,
    val numChannels: Int? = null,
    val albumId: Int,
    val albumName: String,
    val artistId: Int,
    val artistName: String,
    val artUrl: String? = null,
    val liked: Boolean? = null,
) {
    val formattedDuration: String
        get() = formatDuration(duration)

    val displayArtist: String get() = artistName
}

@Serializable
data class TracksResponse(
    val tracks: List<TrackListItem>,
    val total: Int,
    val limit: Int,
    val cursor: Int,
)

@Serializable
data class SignResult(
    val id: Int,
    val url: String,
    @Serializable(with = FlexInstantSerializer::class) val signedAt: Instant,
    @Serializable(with = FlexInstantSerializer::class) val expiresAt: Instant,
)

@Serializable
data class LikedResponse(
    val liked: Boolean,
)

@Serializable
data class PlayHistoryEntry(
    @Serializable(with = FlexInstantSerializer::class) val playedAt: Instant,
    val songId: Int,
    val name: String,
    val duration: Int,
    val albumId: Int,
    val albumName: String,
    val artistId: Int,
    val artistName: String,
    val liked: Boolean,
) {
    val id: Int get() = songId
    val formattedDuration: String
        get() = formatDuration(duration)
}

@Serializable
data class PlaylistSummary(
    val id: Int,
    val name: String,
    val description: String? = null,
    val artPath: String? = null,
    val trackCount: Int,
    @Serializable(with = FlexInstantSerializer::class) val createdAt: Instant,
    @Serializable(with = FlexInstantSerializer::class) val updatedAt: Instant? = null,
)

@Serializable
data class PlaylistTrack(
    val itemId: Int,
    val songId: Int,
    val name: String,
    val duration: Int,
    val number: Int? = null,
    val disc: Int? = null,
    val liked: Boolean? = null,
    val lossless: Boolean? = null,
    val albumId: Int,
    val albumName: String,
    val artistName: String,
    val artUrl: String? = null,
    val prevItemId: Int? = null,
    val nextItemId: Int? = null,
) {
    val id: Int get() = itemId
    val formattedDuration: String
        get() = formatDuration(duration)
}

@Serializable
data class PlaylistDetail(
    val id: Int,
    val name: String,
    val description: String? = null,
    val artPath: String? = null,
    @Serializable(with = FlexInstantSerializer::class) val createdAt: Instant,
    @Serializable(with = FlexInstantSerializer::class) val updatedAt: Instant? = null,
    val tracks: List<PlaylistTrack>,
)

@Serializable
enum class HomeRowType {
    @SerialName("Album") Album,
    @SerialName("Artist") Artist,
    @SerialName("Track") Track,
}

@Serializable
data class HomeRow(
    val name: String,
    val albums: List<AlbumPartial>,
    val rowType: HomeRowType,
    val resource: String? = null,
) {
    val id: String get() = name
}

@Serializable
data class GenreEntry(
    val id: Int,
    val name: String,
    val albumCount: Int,
    val songCount: Int,
)

@Serializable
data class HlsProfile(
    val name: String,
    val codec: String,
    val bitrate: Int? = null,
) {
    val id: String get() = name
    val displayName: String
        get() = if (bitrate != null) {
            "${name.replaceFirstChar { it.uppercase() }} (${bitrate / 1000}k)"
        } else {
            "${name.replaceFirstChar { it.uppercase() }} (Lossless)"
        }
}

@Serializable
data class SearchResult(
    val id: Int,
    val songName: String,
    val artistName: String? = null,
    val albumName: String? = null,
    val picture: String? = null,
)

@Serializable
data class Me(
    val id: Int,
    val name: String? = null,
    val email: String? = null,
    val picture: String? = null,
    val isAdmin: Boolean,
    val lastfmConnected: Boolean,
)

@Serializable
data class LastfmTokenResponse(
    val token: String,
    val url: String,
)

@Serializable
data class LastfmSessionResponse(
    val sessionKey: String,
    val username: String,
)

private fun formatDuration(durationSeconds: Int): String {
    val minutes = durationSeconds / 60
    val seconds = durationSeconds % 60
    return "%d:%02d".format(minutes, seconds)
}
