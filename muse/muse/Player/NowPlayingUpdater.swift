//
//  NowPlayingUpdater.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import Foundation
import AVFoundation
import MediaPlayer
import UIKit

final class NowPlayingUpdater {

    // MARK: - Properties

    private var commandTargets: [Any] = []

    // MARK: - Update

    func update(track: Track, player: AVPlayer) {
        var info: [String: Any] = [
            MPMediaItemPropertyTitle: track.name,
            MPMediaItemPropertyArtist: track.displayArtist,
            MPMediaItemPropertyAlbumTitle: track.albumName,
            MPMediaItemPropertyPlaybackDuration: Double(track.duration),
            MPNowPlayingInfoPropertyPlaybackRate: player.rate,
            MPNowPlayingInfoPropertyElapsedPlaybackTime: player.currentTime().seconds.isNaN ? 0 : player.currentTime().seconds
        ]

        if let year = track.year {
            info[MPMediaItemPropertyAlbumTrackNumber] = year
        }

        MPNowPlayingInfoCenter.default().nowPlayingInfo = info

        if let artUrlString = track.artUrl, let artURL = URL(string: artUrlString) {
            Task {
                await fetchAndSetArtwork(from: artURL)
            }
        }
    }

    private func fetchAndSetArtwork(from url: URL) async {
        guard let (data, _) = try? await URLSession.shared.data(from: url),
              let image = UIImage(data: data) else { return }

        let artwork = MPMediaItemArtwork(boundsSize: image.size) { _ in image }
        var info = MPNowPlayingInfoCenter.default().nowPlayingInfo ?? [:]
        info[MPMediaItemPropertyArtwork] = artwork

        await MainActor.run {
            MPNowPlayingInfoCenter.default().nowPlayingInfo = info
        }
    }

    // MARK: - Remote Commands

    func setupRemoteCommands(
        onPlay: @escaping () -> Void,
        onPause: @escaping () -> Void,
        onNext: @escaping () -> Void,
        onPrevious: @escaping () -> Void,
        onSeek: @escaping (Double) -> Void
    ) {
        let commandCenter = MPRemoteCommandCenter.shared()

        let playTarget = commandCenter.playCommand.addTarget { _ in
            onPlay()
            return .success
        }

        let pauseTarget = commandCenter.pauseCommand.addTarget { _ in
            onPause()
            return .success
        }

        let nextTarget = commandCenter.nextTrackCommand.addTarget { _ in
            onNext()
            return .success
        }

        let prevTarget = commandCenter.previousTrackCommand.addTarget { _ in
            onPrevious()
            return .success
        }

        let seekTarget = commandCenter.changePlaybackPositionCommand.addTarget { event in
            if let posEvent = event as? MPChangePlaybackPositionCommandEvent {
                onSeek(posEvent.positionTime)
            }
            return .success
        }

        commandCenter.playCommand.isEnabled = true
        commandCenter.pauseCommand.isEnabled = true
        commandCenter.nextTrackCommand.isEnabled = true
        commandCenter.previousTrackCommand.isEnabled = true
        commandCenter.changePlaybackPositionCommand.isEnabled = true

        // Hold references so they aren't deallocated
        commandTargets = [playTarget, pauseTarget, nextTarget, prevTarget, seekTarget]
    }

    func removeRemoteCommands() {
        let commandCenter = MPRemoteCommandCenter.shared()
        commandCenter.playCommand.removeTarget(nil)
        commandCenter.pauseCommand.removeTarget(nil)
        commandCenter.nextTrackCommand.removeTarget(nil)
        commandCenter.previousTrackCommand.removeTarget(nil)
        commandCenter.changePlaybackPositionCommand.removeTarget(nil)
        commandTargets = []
    }
}
