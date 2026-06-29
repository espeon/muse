import Foundation
import UIKit

// MARK: - ArtistPartial

extension ArtistPartial {
    static let preview = ArtistPartial(
        id: 1,
        slug: "american-football",
        name: "American Football",
        picture: nil,
        numAlbums: 4
    )
}

// MARK: - AlbumPartial

extension AlbumPartial {
    static let preview = AlbumPartial(
        id: 1,
        slug: "american-football",
        name: "American Football",
        disambiguation: nil,
        art: ["https://picsum.photos/seed/af/500/500"],
        year: 1999,
        count: 9,
        artist: .preview
    )

    static let previewList: [AlbumPartial] = [
        .preview,
        AlbumPartial(id: 2, slug: nil, name: "American Football", disambiguation: nil, art: [], year: 2016, count: 10, artist: .preview),
        AlbumPartial(id: 3, slug: nil, name: "American Football", disambiguation: nil, art: [], year: 2019, count: 11, artist: .preview),
        AlbumPartial(id: 4, slug: nil, name: "American Football", disambiguation: nil, art: [], year: 2023, count: 10, artist: .preview),
    ]
}

// MARK: - Album

extension Album {
    static let preview = Album(
        id: 1,
        name: "American Football",
        art: ["https://picsum.photos/seed/af/500/500"],
        year: 1999,
        genres: ["post-rock", "emo"],
        copyright: nil,
        label: "Polyvinyl Record Co.",
        createdAt: .now,
        updatedAt: nil,
        artist: .preview,
        tracks: Track.previewQueue
    )
}

// MARK: - Track

extension Track {
    static let preview = Track(
        id: 1,
        name: "Never Meant",
        albumArtist: 1,
        artists: [.preview],
        plays: 42,
        duration: 274,
        liked: true,
        lastPlay: nil,
        year: 1999,
        number: 1,
        disc: nil,
        lossless: true,
        sampleRate: 44100,
        bitsPerSample: 16,
        numChannels: 2,
        composer: nil,
        isrc: nil,
        bpm: nil,
        createdAt: .now,
        updatedAt: nil,
        album: 1,
        albumName: "American Football",
        artistName: "American Football",
        artUrl: nil
    )

    static let previewQueue: [Track] = [
        .preview,
        Track(id: 2, name: "The Summer Ends", albumArtist: 1, artists: [.preview],
              plays: 30, duration: 296, liked: false, lastPlay: nil, year: 1999,
              number: 2, disc: nil, lossless: true, sampleRate: 44100, bitsPerSample: 16,
              numChannels: 2, composer: nil, isrc: nil, bpm: nil,
              createdAt: .now, updatedAt: nil,
              album: 1, albumName: "American Football", artistName: "American Football", artUrl: nil),
        Track(id: 3, name: "Honestly?", albumArtist: 1, artists: [.preview],
              plays: 28, duration: 258, liked: nil, lastPlay: nil, year: 1999,
              number: 3, disc: nil, lossless: false, sampleRate: nil, bitsPerSample: nil,
              numChannels: nil, composer: nil, isrc: nil, bpm: nil,
              createdAt: .now, updatedAt: nil,
              album: 1, albumName: "American Football", artistName: "American Football", artUrl: nil),
        Track(id: 4, name: "For Sure", albumArtist: 1, artists: [.preview],
              plays: 19, duration: 214, liked: true, lastPlay: nil, year: 1999,
              number: 4, disc: nil, lossless: true, sampleRate: 48000, bitsPerSample: 24,
              numChannels: 2, composer: nil, isrc: nil, bpm: nil,
              createdAt: .now, updatedAt: nil,
              album: 1, albumName: "American Football", artistName: "American Football", artUrl: nil),
    ]
}

// MARK: - PlaylistSummary

extension PlaylistSummary {
    static let preview = PlaylistSummary(
        id: 1,
        name: "Late Night Emo",
        description: "For when the feelings hit",
        artPath: nil,
        trackCount: 24,
        createdAt: .now,
        updatedAt: nil
    )
}

// MARK: - PlayerEngine

extension PlayerEngine {
    /// A PlayerEngine with a mock queue pre-loaded, for use in previews.
    static var preview: PlayerEngine {
        let engine = PlayerEngine()
        engine.queue = Track.previewQueue
        engine.currentIndex = 0
        engine.isPlaying = true
        engine.duration = Double(Track.preview.duration)
        engine.currentTime = 47
        return engine
    }

    static var previewIdle: PlayerEngine {
        PlayerEngine()
    }
}

// MARK: - RemoteClient

extension RemoteClient {
    /// A `RemoteClient` with a single fake "iPad" device marked as the
    /// active player and a placeholder `lastState`, for previews of
    /// the "playing on other device" UI.
    @MainActor
    static var previewRemote: RemoteClient {
        let client = RemoteClient(
            serverURL: { "https://example.invalid" },
            authHeader: { nil }
        )
        client._setPreviewState(
            connectionState: .connected,
            myDeviceId: "preview-self",
            activeDeviceId: .some("preview-ipad"),
            devices: [
                RemoteDevice(
                    deviceId: "preview-self",
                    name: UIDevice.current.name,
                    kind: .ios,
                    isActivePlayer: false,
                    lastSeen: 0
                ),
                RemoteDevice(
                    deviceId: "preview-ipad",
                    name: "iPad",
                    kind: .ios,
                    isActivePlayer: true,
                    lastSeen: 0
                ),
            ]
        )
        return client
    }

    /// A `RemoteClient` with no other devices — the local device is
    /// the active player. For previews of the "no remote" UI.
    @MainActor
    static var previewLocalActive: RemoteClient {
        let client = RemoteClient(
            serverURL: { "https://example.invalid" },
            authHeader: { nil }
        )
        client._setPreviewState(
            connectionState: .connected,
            myDeviceId: "preview-self",
            activeDeviceId: .some("preview-self"),
            devices: [
                RemoteDevice(
                    deviceId: "preview-self",
                    name: UIDevice.current.name,
                    kind: .ios,
                    isActivePlayer: true,
                    lastSeen: 0
                ),
            ]
        )
        return client
    }
}
