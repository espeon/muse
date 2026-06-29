package vg.nat.muse.ui

import vg.nat.muse.net.PlaylistTrack
import vg.nat.muse.net.SearchResult
import vg.nat.muse.net.Track
import java.time.Instant

fun PlaylistTrack.toTrack(): Track = Track(
    id = songId,
    name = name,
    albumArtist = 0,
    artists = emptyList(),
    duration = duration,
    liked = liked,
    number = number,
    disc = disc,
    lossless = lossless,
    createdAt = Instant.now(),
    album = albumId,
    albumName = albumName,
    artistName = artistName,
    artUrl = artUrl,
)

fun List<PlaylistTrack>.toTracks(): List<Track> = map { it.toTrack() }

fun SearchResult.toTrack(): Track = Track(
    id = id,
    name = songName,
    albumArtist = 0,
    artists = emptyList(),
    duration = 0,
    createdAt = Instant.now(),
    album = 0,
    albumName = albumName ?: "",
    artistName = artistName ?: "",
    artUrl = picture,
)
