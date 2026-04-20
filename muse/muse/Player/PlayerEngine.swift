//
//  PlayerEngine.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import AVFoundation
import Foundation
import MediaPlayer
import Observation
import UIKit

@Observable
final class PlayerEngine: NSObject {

    // MARK: - Public State

    var queue: [Track] = []
    var currentIndex: Int = 0
    var isPlaying: Bool = false
    var currentTime: Double = 0
    var duration: Double = 0
    var volume: Float = 1.0
    var showFullPlayer: Bool = false

    var currentTrack: Track? {
        guard !queue.isEmpty, currentIndex < queue.count else { return nil }
        return queue[currentIndex]
    }

    var useHLS: Bool = UserDefaults.standard.bool(forKey: "muse.useHLS") {
        didSet { UserDefaults.standard.set(useHLS, forKey: "muse.useHLS") }
    }

    var selectedProfile: String? = UserDefaults.standard.string(forKey: "muse.hlsProfile") {
        didSet { UserDefaults.standard.set(selectedProfile, forKey: "muse.hlsProfile") }
    }

    var hlsProfiles: [HLSProfile] = []
    var currentHLSProfile: HLSProfile?

    // MARK: - Private

    private let avPlayer = AVQueuePlayer()
    private var timeObserver: Any?
    private var displayLink: CADisplayLink?
    private var signedURLCache: [Int: String] = [:]
    // Number of upcoming tracks to pre-sign/pre-load
    private let preloadCount = 3
    private var currentApiClient: APIClient?
    private var didFinishObserver: NSObjectProtocol?
    private var lastDetectedProfile: HLSProfile?

    // MARK: - Init

    override init() {
        super.init()
        setupAudioSession()
        setupTimeObserver()
        setupRemoteTransportControls()
        setupFinishObserver()
    }

    // MARK: - Audio Session

    private func setupAudioSession() {
        #if os(iOS)
            do {
                try AVAudioSession.sharedInstance().setCategory(.playback, mode: .default)
                try AVAudioSession.sharedInstance().setActive(true)
            } catch {
                print("[PlayerEngine] Audio session setup failed: \(error)")
            }
        #endif
    }

    // MARK: - Time Observer

    private func setupTimeObserver() {
        let interval = CMTime(seconds: 0.5, preferredTimescale: CMTimeScale(NSEC_PER_SEC))
        timeObserver = avPlayer.addPeriodicTimeObserver(forInterval: interval, queue: .main) {
            [weak self] time in
            guard let self else { return }
            if let item = self.avPlayer.currentItem {
                let d = item.duration.seconds
                if !d.isNaN && !d.isInfinite {
                    self.duration = d
                }
            }
            self.updateCurrentQuality()
        }
    }

    private func updateCurrentQuality() {
        guard useHLS,
            let bitrate = avPlayer.currentItem?.accessLog()?.events.last?.indicatedBitrate,
            bitrate > 0
        else {
            currentHLSProfile = nil
            lastDetectedProfile = nil
            return
        }

        let maxLossyBitrate = hlsProfiles.compactMap(\.bitrate).map(Double.init).max() ?? 0
        let bitrateDouble = Double(bitrate)

        let detected: HLSProfile?
        if bitrateDouble > maxLossyBitrate + 560_000 {
            detected = hlsProfiles.first(where: { $0.bitrate == nil })
        } else {
            detected = hlsProfiles.min(by: {
                abs(Double($0.bitrate ?? .max) - bitrateDouble)
                    < abs(Double($1.bitrate ?? .max) - bitrateDouble)
            })
        }

        if detected?.id == lastDetectedProfile?.id {
            currentHLSProfile = detected
        }
        lastDetectedProfile = detected
    }

    func setQuality(_ profileName: String?) {
        selectedProfile = profileName
        guard let item = avPlayer.currentItem, useHLS else { return }
        item.preferredPeakBitRate = peakBitRate(for: profileName)
    }

    private func peakBitRate(for profileName: String?) -> Double {
        guard let name = profileName,
            let profile = hlsProfiles.first(where: { $0.name == name })
        else {
            return 0
        }
        return Double(profile.bitrate ?? 100_000_000_000)
    }

    private func makePlayerItem(url: URL) -> AVPlayerItem {
        let item = AVPlayerItem(url: url)
        guard useHLS else { return item }

        item.preferredPeakBitRate = peakBitRate(for: selectedProfile)
        item.variantPreferences = .scalabilityToLosslessAudio
        return item
    }

    private func startDisplayLink() {
        guard displayLink == nil else { return }
        let link = CADisplayLink(target: self, selector: #selector(displayLinkTick))
        link.add(to: .main, forMode: .common)
        displayLink = link
    }

    private func stopDisplayLink() {
        displayLink?.invalidate()
        displayLink = nil
    }

    @objc private func displayLinkTick(_ link: CADisplayLink) {
        let time = avPlayer.currentTime().seconds
        currentTime = time.isNaN ? 0 : time
    }

    // MARK: - Finish Observer

    private func setupFinishObserver() {
        didFinishObserver = NotificationCenter.default.addObserver(
            forName: .AVPlayerItemDidPlayToEndTime,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            self?.handleItemFinished()
        }
    }

    private func handleItemFinished() {
        let nextIndex = currentIndex + 1
        if nextIndex < queue.count {
            currentIndex = nextIndex
            if let apiClient = currentApiClient {
                Task {
                    await self.enqueueUpcoming(apiClient: apiClient)
                }
            }
        } else {
            isPlaying = false
            stopDisplayLink()
        }
        updateNowPlaying()
    }

    // MARK: - Playback Control

    func play(tracks: [Track], startingAt index: Int = 0, apiClient: APIClient) async {
        currentApiClient = apiClient
        queue = tracks
        currentIndex = index
        signedURLCache = [:]

        avPlayer.removeAllItems()

        // Sign and enqueue the starting track + upcoming
        let indicesToSign = Array(index..<min(index + preloadCount + 1, tracks.count))
        let trackIds = indicesToSign.map { tracks[$0].id }

        if useHLS && hlsProfiles.isEmpty {
            hlsProfiles = (try? await apiClient.fetchHLSProfiles()) ?? []
        }

        do {
            let results = try await apiClient.batchSignTracks(
                ids: trackIds, mode: useHLS ? "hls" : nil)
            for result in results {
                signedURLCache[result.id] = result.url
            }
        } catch {
            print("[PlayerEngine] Failed to sign tracks: \(error)")
            return
        }

        // Build and enqueue items
        for i in indicesToSign {
            let track = tracks[i]
            guard let urlStr = signedURLCache[track.id],
                let url = URL(string: urlStr)
            else { continue }
            avPlayer.insert(makePlayerItem(url: url), after: nil)
        }

        avPlayer.play()
        isPlaying = true
        startDisplayLink()
        updateNowPlaying()

        Task {
            try? await apiClient.setPlaying(trackId: tracks[index].id)
        }
    }

    func enqueueUpcoming(apiClient: APIClient) async {
        guard !queue.isEmpty else { return }
        let nextStart = currentIndex + avPlayer.items().count
        let end = min(nextStart + preloadCount, queue.count)
        guard nextStart < end else { return }

        let indicesToSign = Array(nextStart..<end)
        let unsigned = indicesToSign.filter { signedURLCache[queue[$0].id] == nil }

        if !unsigned.isEmpty {
            let ids = unsigned.map { queue[$0].id }
            do {
                let results = try await apiClient.batchSignTracks(
                    ids: ids, mode: useHLS ? "hls" : nil)
                for result in results {
                    signedURLCache[result.id] = result.url
                }
            } catch {
                print("[PlayerEngine] Failed to pre-sign upcoming tracks: \(error)")
                return
            }
        }

        for i in indicesToSign {
            let track = queue[i]
            guard let urlStr = signedURLCache[track.id],
                let url = URL(string: urlStr)
            else { continue }
            avPlayer.insert(makePlayerItem(url: url), after: nil)
        }
    }

    func togglePlayPause() {
        if isPlaying {
            avPlayer.pause()
            isPlaying = false
            stopDisplayLink()
        } else {
            avPlayer.play()
            isPlaying = true
            startDisplayLink()
        }
        updateNowPlaying()
    }

    func next() {
        guard currentIndex + 1 < queue.count else { return }
        avPlayer.advanceToNextItem()
        currentIndex += 1
        updateNowPlaying()
        if let apiClient = currentApiClient {
            Task {
                await enqueueUpcoming(apiClient: apiClient)
                if let track = currentTrack {
                    try? await apiClient.setPlaying(trackId: track.id)
                }
            }
        }
    }

    func previous() {
        if currentTime > 3 {
            seek(to: 0)
            return
        }

        guard currentIndex > 0 else {
            seek(to: 0)
            return
        }

        // Rebuild the queue from the previous index
        if let apiClient = currentApiClient {
            let newIndex = currentIndex - 1
            let tracks = queue
            Task {
                await play(tracks: tracks, startingAt: newIndex, apiClient: apiClient)
            }
        }
    }

    func seek(to time: Double) {
        let cmTime = CMTime(seconds: time, preferredTimescale: CMTimeScale(NSEC_PER_SEC))
        avPlayer.seek(to: cmTime, toleranceBefore: .zero, toleranceAfter: .zero)
        currentTime = time
    }

    func setVolume(_ vol: Float) {
        volume = vol
        avPlayer.volume = vol
    }

    // MARK: - Now Playing Info

    private func updateNowPlaying() {
        guard let track = currentTrack else {
            MPNowPlayingInfoCenter.default().nowPlayingInfo = nil
            return
        }

        var info: [String: Any] = [
            MPMediaItemPropertyTitle: track.name,
            MPMediaItemPropertyArtist: track.displayArtist,
            MPMediaItemPropertyAlbumTitle: track.albumName,
            MPNowPlayingInfoPropertyElapsedPlaybackTime: currentTime,
            MPMediaItemPropertyPlaybackDuration: Double(track.duration),
            MPNowPlayingInfoPropertyPlaybackRate: isPlaying ? 1.0 : 0.0,
        ]

        MPNowPlayingInfoCenter.default().nowPlayingInfo = info

        // Load artwork asynchronously
        if let artUrl = track.artUrl, let url = URL(string: artUrl) {
            Task {
                if let (data, _) = try? await URLSession.shared.data(from: url),
                    let image = UIImage(data: data)
                {
                    let artwork = MPMediaItemArtwork(boundsSize: image.size) { _ in image }
                    var updatedInfo = MPNowPlayingInfoCenter.default().nowPlayingInfo ?? [:]
                    updatedInfo[MPMediaItemPropertyArtwork] = artwork
                    await MainActor.run {
                        MPNowPlayingInfoCenter.default().nowPlayingInfo = updatedInfo
                    }
                }
            }
        }
    }

    // MARK: - Remote Commands

    private func setupRemoteTransportControls() {
        let commandCenter = MPRemoteCommandCenter.shared()

        commandCenter.playCommand.isEnabled = true
        commandCenter.playCommand.addTarget { [weak self] _ in
            self?.avPlayer.play()
            self?.isPlaying = true
            self?.updateNowPlaying()
            return .success
        }

        commandCenter.pauseCommand.isEnabled = true
        commandCenter.pauseCommand.addTarget { [weak self] _ in
            self?.avPlayer.pause()
            self?.isPlaying = false
            self?.updateNowPlaying()
            return .success
        }

        commandCenter.togglePlayPauseCommand.isEnabled = true
        commandCenter.togglePlayPauseCommand.addTarget { [weak self] _ in
            self?.togglePlayPause()
            return .success
        }

        commandCenter.nextTrackCommand.isEnabled = true
        commandCenter.nextTrackCommand.addTarget { [weak self] _ in
            self?.next()
            return .success
        }

        commandCenter.previousTrackCommand.isEnabled = true
        commandCenter.previousTrackCommand.addTarget { [weak self] _ in
            self?.previous()
            return .success
        }

        commandCenter.changePlaybackPositionCommand.isEnabled = true
        commandCenter.changePlaybackPositionCommand.addTarget { [weak self] event in
            if let positionEvent = event as? MPChangePlaybackPositionCommandEvent {
                self?.seek(to: positionEvent.positionTime)
            }
            return .success
        }
    }

    deinit {
        stopDisplayLink()
        if let observer = timeObserver {
            avPlayer.removeTimeObserver(observer)
        }
        if let observer = didFinishObserver {
            NotificationCenter.default.removeObserver(observer)
        }
    }
}
