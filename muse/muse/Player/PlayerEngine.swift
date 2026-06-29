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

    /// Stable id of the currently-playing queue item. Mirrors the
    /// server-side `current_item_id` in the remote protocol. Player
    /// mints these on every queue replacement; controllers use them to
    /// address the item in `removeFromQueue` / `reorderQueue`.
    var currentItemId: String? {
        guard currentIndex >= 0, currentIndex < queueItemIds.count else { return nil }
        return queueItemIds[currentIndex]
    }

    /// Whether this device is currently the active player in the remote
    /// session. Set by the RemoteClient via `handleActivePlayerChange`.
    /// Read freely from any context.
    private(set) var isActivePlayer: Bool = false

    /// Weak ref to the remote client. Set once during app composition.
    /// When set, the engine publishes state to the server on every
    /// change and while playing.
    weak var remote: RemoteClient?

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

    /// Parallel to `queue`: the player-minted item_id for each track.
    /// Indexed in lockstep with `queue`.
    private var queueItemIds: [String] = []

    /// Periodic publisher while this device is the active player and is
    /// playing. Keeps the server-cached `position_ms` fresh.
    private var publishTask: Task<Void, Never>?

    // MARK: - Init

    override init() {
        super.init()
        setupAudioSession()
        setupTimeObserver()
        setupRemoteTransportControls()
        setupFinishObserver()
        startPeriodicPublish()
    }

    // MARK: - Remote session hooks

    /// Called by `RemoteClient` when this device's active-player status
    /// changes. When we become inactive, local playback is paused so
    /// audio stops here (the active device takes over). The local queue
    /// is preserved so the user can resume manually.
    func handleActivePlayerChange(_ isActive: Bool) {
        let prev = isActivePlayer
        isActivePlayer = isActive
        if prev && !isActive {
            // Another device took over. Stop local audio.
            avPlayer.pause()
            isPlaying = false
            stopDisplayLink()
            updateNowPlaying()
        }
        if !prev && isActive {
            // We just became the active player. Publish our current
            // state so the new state is visible to other devices.
            publishIfActive()
        }
    }

    /// Execute a command routed from a controller. Only invoked when
    /// this device is the active player.
    func handleRemoteCommand(_ command: RemoteCommand) {
        switch command.kind {
        case .play:
            if !isPlaying { togglePlayPause() }
        case .pause:
            if isPlaying { togglePlayPause() }
        case .toggle:
            togglePlayPause()
        case .next:
            next()
        case .previous:
            previous()
        case .seek:
            if let ms = command.positionMs {
                seek(to: Double(ms) / 1000.0)
            }
        case .setQueue:
            if let ids = command.trackIds {
                let start = command.startIndex ?? 0
                Task { @MainActor [weak self] in
                    await self?.playQueue(trackIds: ids, startIndex: start)
                }
            }
        case .addToQueue:
            if let id = command.trackId {
                Task { @MainActor [weak self] in
                    await self?.addTrackById(trackId: id, afterItemId: command.afterItemId)
                }
            }
        case .removeFromQueue:
            if let itemId = command.itemId {
                removeFromQueue(itemId: itemId)
            }
        case .reorderQueue:
            if let itemId = command.itemId {
                reorderQueue(itemId: itemId, afterItemId: command.afterItemId)
            }
        }
    }

    /// Local queue edit: insert a track into the queue at the position
    /// after `afterItemId` (or append if nil). Also enqueues to the
    /// AVQueuePlayer at the end (AVQueuePlayer does not support
    /// mid-queue insertion; the new track will play at the end of the
    /// avPlayer's local order, but the model — and the server's view —
    /// reflects the requested position).
    func addToQueue(track: Track, afterItemId: String? = nil) {
        let newItemId = UUID().uuidString.lowercased()
        let insertionIndex: Int
        if let afterId = afterItemId,
            let idx = queueItemIds.firstIndex(of: afterId)
        {
            insertionIndex = idx + 1
        } else {
            insertionIndex = queue.count
        }
        queue.insert(track, at: insertionIndex)
        queueItemIds.insert(newItemId, at: insertionIndex)

        // Try to sign and enqueue to the live avPlayer. If signing
        // fails the model is still correct; the track will play when
        // the current one ends (avPlayer advances to its next item).
        if let apiClient = currentApiClient {
            Task { [weak self] in
                guard let self else { return }
                if let urlStr = try? await apiClient.signTrack(id: track.id).url,
                    let url = URL(string: urlStr)
                {
                    await MainActor.run {
                        self.avPlayer.insert(self.makePlayerItem(url: url), after: nil)
                    }
                }
                self.publishIfActive()
            }
        } else {
            publishIfActive()
        }
    }

    /// Local queue edit: remove an item. If it's the current item,
    /// advance to the next (or stop if at end).
    func removeFromQueue(itemId: String) {
        guard let idx = queueItemIds.firstIndex(of: itemId) else { return }
        let isCurrent = idx == currentIndex

        queue.remove(at: idx)
        queueItemIds.remove(at: idx)

        if isCurrent {
            if currentIndex < queue.count {
                avPlayer.advanceToNextItem()
            } else {
                avPlayer.pause()
                isPlaying = false
                stopDisplayLink()
            }
        }
        publishIfActive()
    }

    /// Local queue edit: move an item to after `afterItemId` (or to the
    /// start if nil). The avPlayer order is not updated; the model's
    /// order is what other devices see, and a manual skip will reveal
    /// the divergence in the local avPlayer.
    func reorderQueue(itemId: String, afterItemId: String?) {
        guard let from = queueItemIds.firstIndex(of: itemId) else { return }
        let track = queue[from]
        let id = queueItemIds[from]
        queue.remove(at: from)
        queueItemIds.remove(at: from)

        let to: Int
        if let afterId = afterItemId,
            let idx = queueItemIds.firstIndex(of: afterId)
        {
            to = idx + 1
        } else {
            to = 0
        }
        // If we removed before the insertion point, the target index
        // shifts left by one.
        let adjusted = from < to ? to - 1 : to
        queue.insert(track, at: adjusted)
        queueItemIds.insert(id, at: adjusted)

        if currentIndex == from {
            currentIndex = adjusted
        } else if from < currentIndex, currentIndex <= adjusted {
            currentIndex -= 1
        } else if from > currentIndex, adjusted <= currentIndex {
            currentIndex += 1
        }
        publishIfActive()
    }

    /// Replace the queue with the given track ids, starting at the
    /// given index. Fetches each track via the API.
    private func playQueue(trackIds: [Int], startIndex: Int) async {
        guard let apiClient = currentApiClient else { return }
        do {
            let tracks = try await fetchTracks(ids: trackIds, apiClient: apiClient)
            let safeIndex = max(0, min(startIndex, tracks.count - 1))
            await play(tracks: tracks, startingAt: safeIndex, apiClient: apiClient)
        } catch {
            print("[PlayerEngine] playQueue fetch failed: \(error)")
        }
    }

    private func addTrackById(trackId: Int, afterItemId: String?) async {
        guard let apiClient = currentApiClient else { return }
        do {
            let track = try await apiClient.fetchTrack(id: trackId)
            addToQueue(track: track, afterItemId: afterItemId)
        } catch {
            print("[PlayerEngine] addTrackById fetch failed: \(error)")
        }
    }

    private func fetchTracks(ids: [Int], apiClient: APIClient) async throws -> [Track] {
        // Sequential for now; the existing API has no batch fetch for
        // arbitrary tracks by id. Could be optimised later.
        var out: [Track] = []
        for id in ids {
            out.append(try await apiClient.fetchTrack(id: id))
        }
        return out
    }

    // MARK: - State publish to remote

    private func startPeriodicPublish() {
        publishTask?.cancel()
        publishTask = Task { [weak self] in
            while !Task.isCancelled {
                try? await Task.sleep(nanoseconds: 5 * 1_000_000_000)
                guard !Task.isCancelled else { return }
                self?.publishIfActive()
            }
        }
    }

    private func publishIfActive() {
        guard isActivePlayer, let remote else { return }
        let state = buildRemoteState()
        Task { @MainActor [weak remote] in
            await remote?.publishState(state)
        }
    }

    private func buildRemoteState() -> RemotePlaybackState {
        RemotePlaybackState(
            currentItemId: currentItemId,
            positionMs: Int64(currentTime * 1000),
            isPlaying: isPlaying,
            queue: zip(queue, queueItemIds).map { RemoteQueueItem(itemId: $1, trackId: $0.id) },
            updatedAt: Int64(Date().timeIntervalSince1970 * 1000)
        )
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
        publishIfActive()
    }

    // MARK: - Playback Control

    func play(tracks: [Track], startingAt index: Int = 0, apiClient: APIClient) async {
        currentApiClient = apiClient
        queue = tracks
        currentIndex = index
        // Player mints stable item_ids for each track in the new queue.
        queueItemIds = tracks.map { _ in UUID().uuidString.lowercased() }
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
        publishIfActive()
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
        publishIfActive()
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
        publishIfActive()
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
        publishIfActive()
    }

    func seek(to time: Double) {
        let cmTime = CMTime(seconds: time, preferredTimescale: CMTimeScale(NSEC_PER_SEC))
        avPlayer.seek(to: cmTime, toleranceBefore: .zero, toleranceAfter: .zero)
        currentTime = time
        publishIfActive()
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
        publishTask?.cancel()
        stopDisplayLink()
        if let observer = timeObserver {
            avPlayer.removeTimeObserver(observer)
        }
        if let observer = didFinishObserver {
            NotificationCenter.default.removeObserver(observer)
        }
    }
}
