import SwiftUI
import Translation

struct NowPlayingSheetView: View {
    @Bindable var player: PlayerEngine
    let remote: RemoteClient
    var dismiss: () -> Void

    @State private var showLyrics = false
    @State private var lyrics: JLF?
    @State private var isDraggingProgress = false
    @State private var progressDragValue: Double = 0
    @State private var likedState: Bool? = nil
    @State private var isTogglingLike = false
    @State private var showQueue = false

    @State private var translations: [String]? = nil
    @State private var bgVoxTranslations: [String]? = nil
    @State private var showTranslationPicker = false
    @State private var selectedTranslationLanguage: String? = nil
    @State private var translationConfig: TranslationSession.Configuration? = nil
    @State private var useLLMTranslation = false
    @State private var llmTargetLanguage = ""
    @StateObject private var translationService = TranslationService()

    // Track metadata cache for remote state. Remote state has only
    // `track_id` values; we look up the full `Track` here so the view
    // can render artwork, title, artist, etc.
    @State private var remoteTrackCache: [Int: Track] = [:]
    @State private var fetchingTrackIds: Set<Int> = []

    @Environment(\.umiClient) private var umiClient
    @Environment(\.apiClient) private var apiClient

    // MARK: - Display source

    /// True when another device is the active player and the controls
    /// should be routed to that device rather than the local engine.
    private var isRemote: Bool {
        guard let active = remote.activeDeviceId else { return false }
        return active != remote.myDeviceId
    }

    private var displayTrack: Track? {
        if isRemote {
            guard let state = remote.lastState,
                let itemId = state.currentItemId,
                let item = state.queue.first(where: { $0.itemId == itemId })
            else { return nil }
            return remoteTrackCache[item.trackId]
        }
        return player.currentTrack
    }

    private var displayIsPlaying: Bool {
        isRemote ? (remote.lastState?.isPlaying ?? false) : player.isPlaying
    }

    private var displayCurrentTime: Double {
        isRemote ? Double(remote.lastState?.positionMs ?? 0) / 1000.0 : player.currentTime
    }

    private var displayDuration: Double {
        if isRemote {
            return Double(displayTrack?.duration ?? 0)
        }
        return player.duration
    }

    private var displayQueue: [Track] {
        if isRemote {
            return (remote.lastState?.queue ?? []).compactMap { remoteTrackCache[$0.trackId] }
        }
        return player.queue
    }

    private var displayQueueIndex: Int? {
        if isRemote {
            guard let state = remote.lastState,
                let itemId = state.currentItemId,
                let idx = state.queue.firstIndex(where: { $0.itemId == itemId })
            else { return nil }
            return idx
        }
        if player.queue.indices.contains(player.currentIndex) {
            return player.currentIndex
        }
        return nil
    }

    // MARK: - Body

    var body: some View {
        GeometryReader { geo in
            ZStack {
                artworkBackground

                VStack(spacing: 0) {

                    Button(action: dismiss) {
                        RoundedRectangle(cornerRadius: 999)
                            .fill(Color.gray.opacity(0.45))
                            .frame(maxWidth: 60, maxHeight: 7.5)
                    }
                    .frame(maxWidth: .infinity)
                    .padding(.top, 48)
                    .safeAreaPadding(.top)

                    Spacer().frame(minHeight: 8)

                    ZStack {
                        artworkContent(geo: geo)
                            .opacity(showLyrics ? 0 : 1)
                            .scaleEffect(showLyrics ? 0.8 : 1)

                        if showLyrics {
                            lyricsView
                                .transition(.opacity.combined(with: .scale(scale: 0.95)))
                        }

                        if showLyrics {
                            VStack {
                                trackInfo(compact: true)
                                    .padding(.top, 24)
                                    .padding(.horizontal, 10)
                                    .safeAreaPadding(.top)
                                Spacer()
                            }
                            .transition(
                                .asymmetric(
                                    insertion: .move(edge: .top).combined(with: .opacity),
                                    removal: .move(edge: .top).combined(with: .opacity)
                                ))
                        }
                    }
                    .frame(height: showLyrics ? geo.size.height * 0.72 : nil)
                    .animation(.spring(response: 0.4, dampingFraction: 0.8), value: showLyrics)

                    Spacer().frame(minHeight: 8, maxHeight: 24)

                    controls(compact: showLyrics)
                        .padding(.top, showLyrics ? 6 : 10)
                        .padding(.bottom, showLyrics ? 80 : 25)
                        .animation(.spring(response: 0.4, dampingFraction: 0.8), value: showLyrics)
                }
                .padding(.top, geo.safeAreaInsets.top)
                .padding(.bottom, geo.safeAreaInsets.bottom)
                .frame(width: geo.size.width, height: geo.size.height)
            }
            .frame(width: geo.size.width, height: geo.size.height)
            .clipped()
        }
        .ignoresSafeArea()
        .safeAreaInset(edge: .top, spacing: 0) {
            if isRemote {
                remoteHeader
                    .padding(.horizontal, 16)
                    .padding(.top, 4)
                    .transition(.move(edge: .top).combined(with: .opacity))
            }
        }
        .animation(.spring(response: 0.4, dampingFraction: 0.8), value: isRemote)
        .sheet(isPresented: $showQueue) {
            NowPlayingQueueSheet(
                tracks: displayQueue,
                currentTrackId: displayTrack?.id
            )
        }
        .sheet(isPresented: $showTranslationPicker) {
            TranslationLanguagePickerView(
                selectedLanguage: $selectedTranslationLanguage,
                useLLM: $useLLMTranslation,
                llmLanguage: $llmTargetLanguage,
                llmConfigured: LLMTranslationConfig.current.isConfigured,
                onSelect: { triggerTranslation() }
            )
        }
        .translationTask(translationConfig) { session in
            Task { @MainActor in
                await performTranslation(using: session)
            }
        }
        .onAppear {
            likedState = displayTrack?.liked
            loadLyrics()
            if isRemote { Task { await prefetchRemoteTracks() } }
        }
        .onChange(of: displayTrack?.id) { _, _ in
            likedState = displayTrack?.liked
            loadLyrics()
        }
        .onChange(of: isRemote) { _, newValue in
            if newValue {
                Task { await prefetchRemoteTracks() }
            } else {
                // We just became local active; nothing to do — local
                // state takes over automatically.
            }
        }
        .onChange(of: remote.lastState?.currentItemId) { _, newId in
            guard isRemote,
                let itemId = newId,
                let state = remote.lastState,
                let item = state.queue.first(where: { $0.itemId == itemId })
            else { return }
            let trackId = item.trackId
            if remoteTrackCache[trackId] == nil {
                Task { await fetchTrack(id: trackId) }
            }
        }
        .onChange(of: remote.lastState?.queue) { _, _ in
            guard isRemote else { return }
            Task { await prefetchRemoteTracks() }
        }
    }

    // MARK: - Remote header

    @ViewBuilder
    private var remoteHeader: some View {
        HStack(spacing: 10) {
            Image(systemName: "speaker.wave.3.fill")
                .foregroundStyle(.green)
            VStack(alignment: .leading, spacing: 1) {
                Text("Playing on \(activeDeviceName)")
                    .font(.subheadline)
                    .foregroundStyle(.white)
                if let state = remote.lastState, !state.isPlaying {
                    Text("Paused")
                        .font(.caption)
                        .foregroundStyle(.white.opacity(0.7))
                }
            }
            Spacer()
            Button {
                Task { await remote.sendTransfer(to: remote.myDeviceId) }
            } label: {
                Label("Play here", systemImage: "iphone.gen3")
                    .font(.subheadline)
                    .padding(.horizontal, 12)
                    .padding(.vertical, 6)
                    .background(.green.opacity(0.25), in: Capsule())
                    .foregroundStyle(.white)
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 8)
        .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 14))
    }

    private var activeDeviceName: String {
        if let id = remote.activeDeviceId,
            let device = remote.devices.first(where: { $0.deviceId == id })
        {
            return device.name
        }
        return "another device"
    }

    // MARK: - Background

    private var artworkBackground: some View {
        Group {
            if let artUrl = displayTrack?.artUrl, let url = URL(string: artUrl) {
                DynamicArtworkBackground(url: url, move: true)
            } else {
                Color.secondary.opacity(0.5)
            }
        }
        .ignoresSafeArea()
    }

    // MARK: - Artwork + track info

    private func artworkContent(geo: GeometryProxy) -> some View {
        let maxWidth = geo.size.width - 44
        let maxHeight = geo.size.height * 0.50
        let baseSize = min(maxWidth, maxHeight, 420)
        let reduced: CGFloat = 40
        let playingScale = displayIsPlaying ? 1.0 : max(0.0, (baseSize - reduced) / baseSize)

        return VStack(spacing: 32) {
            AnimatedArtworkCard(
                url: displayTrack?.artUrl.flatMap { URL(string: $0) },
                queueItemId: displayTrack?.id.description,
                queueIndex: displayQueueIndex,
                size: baseSize,
                cornerRadius: 20,
                playingScale: playingScale
            )

            HStack(spacing: 4) {
                trackInfo(compact: false)
                Spacer()
                Button {
                    toggleLike()
                } label: {
                    Image(systemName: currentLiked ? "heart.fill" : "heart")
                        .font(.title2)
                        .foregroundStyle(currentLiked ? .pink : .white.opacity(0.7))
                }
                .disabled(isTogglingLike)
            }
            .padding(.horizontal, 24)
            .padding(.top, 10)
        }
    }

    // MARK: - Track info

    @ViewBuilder
    private func trackInfo(compact: Bool) -> some View {
        if compact {
            HStack(spacing: 12) {
                ArtworkImage(
                    url: displayTrack?.artUrl,
                    size: 56,
                    cornerRadius: 8
                )
                .shadow(color: .black.opacity(0.3), radius: 8, x: 0, y: 4)

                VStack(alignment: .leading, spacing: 4) {
                    MarqueeText(
                        text: displayTrack?.name ?? "Nothing Playing",
                        font: UIFont.preferredFont(forTextStyle: .headline),
                        leftFade: 16, rightFade: 16, startDelay: 3
                    )
                    .foregroundStyle(.white)

                    MarqueeText(
                        text: displayTrack?.displayArtist ?? "",
                        font: UIFont.preferredFont(forTextStyle: .subheadline),
                        leftFade: 16, rightFade: 16, startDelay: 3
                    )
                    .foregroundStyle(.white.opacity(0.8))
                }

                Spacer()
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 12)
        } else {
            VStack(alignment: .leading, spacing: 4) {
                MarqueeText(
                    text: displayTrack?.name ?? "Nothing Playing",
                    font: UIFont.systemFont(ofSize: 24, weight: .bold),
                    leftFade: 16, rightFade: 16, startDelay: 3
                )
                .foregroundStyle(.white)

                MarqueeText(
                    text: displayTrack?.displayArtist ?? "",
                    font: UIFont.systemFont(ofSize: 20, weight: .medium),
                    leftFade: 16, rightFade: 16, startDelay: 3
                )
                .foregroundStyle(.white.opacity(0.8))
            }
        }
    }

    // MARK: - Lyrics

    @ViewBuilder
    private var lyricsView: some View {
        if let lyrics = lyrics {
            if let richsync = lyrics.richsync {
                RichLyricsView(
                    richsync: richsync,
                    currentTimeMs: Int(displayCurrentTime * 1000),
                    onSeek: { ms in onSeek(ms: ms) },
                    fontDesign: .default,
                    fontWeight: .bold,
                    fontSizeMultiplier: 1.0,
                    fadeCompletedLines: false,
                    translations: translations,
                    bgVoxTranslations: bgVoxTranslations
                )
                .padding(.top, 76)
            } else {
                SyncedLyricsView(
                    lyrics: lyrics,
                    currentTimeMs: Int(displayCurrentTime * 1000),
                    onSeek: { ms in onSeek(ms: ms) },
                    translations: translations
                )
                .padding(.top, 76)
            }
        } else {
            VStack {
                Spacer()
                Image(systemName: "music.note")
                    .font(.system(size: 60))
                    .foregroundStyle(.white.opacity(0.5))
                Text("No lyrics available")
                    .font(.headline)
                    .foregroundStyle(.white.opacity(0.7))
                Spacer()
            }
            .padding(.top, 76)
        }
    }

    // MARK: - Controls

    private func controls(compact: Bool) -> some View {
        VStack(spacing: compact ? 32 : 45) {
            progressBar
                .padding(.horizontal, 24)

            HStack(spacing: 48) {
                Button {
                    onPrevious()
                } label: {
                    Image(systemName: "backward.fill")
                        .font(.largeTitle)
                        .foregroundStyle(.white)
                }

                Button {
                    onTogglePlayPause()
                } label: {
                    Image(systemName: displayIsPlaying ? "pause.fill" : "play.fill")
                        .font(.system(size: 54))
                        .foregroundStyle(.white)
                }

                Button {
                    onNext()
                } label: {
                    Image(systemName: "forward.fill")
                        .font(.largeTitle)
                        .foregroundStyle(.white)
                }
            }
            .padding(.horizontal, 24)

            bottomBar
        }
    }

    // MARK: - Progress bar

    private var progressBar: some View {
        let currentTime = isDraggingProgress ? progressDragValue : displayCurrentTime
        let duration = max(displayDuration, 1)
        let progress = currentTime / duration

        return VStack(spacing: 6) {
            GeometryReader { geo in
                VStack(spacing: 8) {
                    ZStack(alignment: .leading) {
                        RoundedRectangle(cornerRadius: 999)
                            .fill(Color.white.opacity(0.3))
                            .frame(height: 6)

                        RoundedRectangle(cornerRadius: 999)
                            .fill(Color.white)
                            .frame(
                                width: geo.size.width * CGFloat(min(max(progress, 0), 1)), height: 6
                            )
                    }
                    HStack {
                        Text(formatTime(currentTime)).monospaced()
                        Spacer()
                        // The HLS / lossless indicators are a property
                        // of the local playback pipeline; when we're
                        // controlling a remote device, we don't know
                        // its quality, so hide them.
                        if !isRemote {
                            if player.useHLS, let profile = player.currentHLSProfile {
                                HStack(spacing: 4) {
                                    if profile.codec == "flac" {
                                        Text("HI-RES")
                                            .fontWeight(.semibold)
                                            .foregroundStyle(Color.accentColor)
                                        if let track = displayTrack,
                                            let sr = track.sampleRate, let bits = track.bitsPerSample
                                        {
                                            Text("·")
                                            Text("\(bits)-bit \(sr / 1000)kHz FLAC")
                                        }
                                    } else if let bitrate = profile.bitrate {
                                        Text("\(bitrate / 1000) KBPS \(profile.codec.uppercased())")
                                            .fontWeight(.semibold)
                                    }
                                }
                            } else if let track = displayTrack, track.lossless == true {
                                HStack(spacing: 4) {
                                    Text("HI-RES")
                                        .fontWeight(.semibold)
                                        .foregroundStyle(Color.accentColor)
                                    if let sr = track.sampleRate, let bits = track.bitsPerSample {
                                        Text("·")
                                        Text("\(bits)-bit \(sr / 1000)kHz")
                                    }
                                }
                            }
                        }
                        Spacer()
                        Text(formatTime(duration)).monospaced()
                    }
                    .font(.caption)
                    .monospaced()
                    .foregroundStyle(.white.opacity(0.8))
                }
                .contentShape(Rectangle())
                .gesture(
                    DragGesture(minimumDistance: 0)
                        .onChanged { val in
                            isDraggingProgress = true
                            let pct = val.location.x / geo.size.width
                            progressDragValue = max(0, min(Double(pct) * duration, duration))
                        }
                        .onEnded { _ in
                            onSeek(seconds: progressDragValue)
                            isDraggingProgress = false
                        }
                )
            }
            .frame(height: 32)
        }
    }

    // MARK: - Bottom bar

    private var bottomBar: some View {
        HStack(spacing: 0) {
            Button {
                withAnimation(.spring(response: 0.4, dampingFraction: 0.8)) {
                    showLyrics.toggle()
                }
            } label: {
                Image(systemName: "quote.bubble")
                    .font(.title2)
                    .foregroundStyle(showLyrics ? Color.accentColor : .white.opacity(0.7))
                    .frame(maxWidth: 52)
                    .padding(.vertical, 12)
                    .background(
                        Circle().fill(showLyrics ? Color.teal.opacity(0.2) : Color.clear)
                    )
            }

            if showLyrics {
                Button {
                    if translationService.isTranslating { return }
                    if translations != nil {
                        translations = nil
                        bgVoxTranslations = nil
                    } else {
                        showTranslationPicker = true
                    }
                } label: {
                    ZStack {
                        if translationService.isTranslating {
                            ProgressView()
                                .tint(.white)
                        } else {
                            Image(
                                systemName: translations != nil
                                    ? "character.bubble.fill" : "character.bubble"
                            )
                            .font(.title2)
                            .foregroundStyle(translations != nil ? Color.accentColor : .white.opacity(0.7))
                        }
                    }
                    .frame(maxWidth: 52)
                    .padding(.vertical, 12)
                    .background(
                        Circle().fill(
                            translations != nil || translationService.isTranslating
                                ? Color.teal.opacity(0.2) : Color.clear
                        )
                    )
                }
                .disabled(translationService.isTranslating)
                .transition(.opacity.combined(with: .scale(scale: 0.8)))
            }

            Spacer()

            Button {
                showQueue = true
            } label: {
                Image(systemName: "list.bullet")
                    .font(.title2)
                    .foregroundStyle(.white.opacity(0.7))
                    .padding(.vertical, 12)
            }
        }
        .padding(.horizontal, 24)
        .animation(.spring(response: 0.4, dampingFraction: 0.8), value: showLyrics)
    }

    // MARK: - Action dispatch (local or remote)

    private func onTogglePlayPause() {
        if isRemote {
            Task { await remote.sendCommand(.toggle) }
        } else {
            player.togglePlayPause()
        }
    }

    private func onNext() {
        if isRemote {
            Task { await remote.sendCommand(.next) }
        } else {
            player.next()
        }
    }

    private func onPrevious() {
        if isRemote {
            Task { await remote.sendCommand(.previous) }
        } else {
            player.previous()
        }
    }

    private func onSeek(ms: Int) {
        if isRemote {
            Task { await remote.sendCommand(.seek(positionMs: Int64(ms))) }
        } else {
            player.seek(to: Double(ms) / 1000.0)
        }
    }

    private func onSeek(seconds: Double) {
        onSeek(ms: Int(seconds * 1000))
    }

    // MARK: - Remote track fetching

    private func prefetchRemoteTracks() async {
        guard let state = remote.lastState else { return }
        for item in state.queue {
            if remoteTrackCache[item.trackId] == nil {
                await fetchTrack(id: item.trackId)
            }
        }
    }

    private func fetchTrack(id: Int) async {
        if remoteTrackCache[id] != nil { return }
        if fetchingTrackIds.contains(id) { return }
        fetchingTrackIds.insert(id)
        defer { fetchingTrackIds.remove(id) }
        guard let track = try? await apiClient.fetchTrack(id: id) else { return }
        await MainActor.run {
            remoteTrackCache[id] = track
        }
    }

    // MARK: - Helpers

    private var currentLiked: Bool {
        likedState ?? displayTrack?.liked ?? false
    }

    private func formatTime(_ seconds: Double) -> String {
        let s = Int(max(0, seconds))
        return String(format: "%d:%02d", s / 60, s % 60)
    }

    private func toggleLike() {
        guard let track = displayTrack, !isTogglingLike else { return }
        isTogglingLike = true
        likedState = !(likedState ?? track.liked ?? false)
        Task {
            do {
                let result = try await apiClient.toggleLike(trackId: track.id)
                await MainActor.run { likedState = result.liked }
            } catch {
                await MainActor.run { likedState = track.liked }
            }
            await MainActor.run { isTogglingLike = false }
        }
    }

    private func triggerTranslation() {
        guard lyrics != nil else { return }
        translations = nil
        bgVoxTranslations = nil

        if useLLMTranslation {
            Task { await performLLMTranslation() }
        } else {
            if translationConfig == nil {
                if let langCode = selectedTranslationLanguage {
                    translationConfig = TranslationSession.Configuration(
                        source: nil,
                        target: Locale.Language(identifier: langCode)
                    )
                } else {
                    translationConfig = TranslationSession.Configuration()
                }
            } else {
                translationConfig?.target = selectedTranslationLanguage
                    .map { Locale.Language(identifier: $0) }
                translationConfig?.invalidate()
            }
        }
    }

    private func performLLMTranslation() async {
        guard let lyrics, let trackId = displayTrack?.id else { return }

        let mainTexts: [String]
        let bgTexts: [String]

        if let richsync = lyrics.richsync {
            let lines = richsync.sections.flatMap(\.lines)
            mainTexts = lines.map(\.text)
            bgTexts = lines.map { $0.bgVox?.text ?? "" }
        } else {
            mainTexts = lyrics.lines.lines.map(\.text)
            bgTexts = []
        }

        let allTexts = mainTexts + bgTexts

        if let cached = TranslationCache.get(trackId: trackId, texts: allTexts, target: llmTargetLanguage, method: "llm") {
            applyTranslations(cached, mainCount: mainTexts.count, bgCount: bgTexts.count)
            return
        }

        translationService.isTranslating = true
        let config = LLMTranslationConfig.current

        do {
            let allTranslated = try await LLMTranslationService().translate(
                texts: allTexts,
                targetLanguage: llmTargetLanguage,
                config: config
            )
            applyTranslations(allTranslated, mainCount: mainTexts.count, bgCount: bgTexts.count)
            TranslationCache.set(allTranslated, trackId: trackId, texts: allTexts, target: llmTargetLanguage, method: "llm")
        } catch {
            print("[LLM Translation] failed: \(error)")
        }
        translationService.isTranslating = false
    }

    private func performTranslation(using session: TranslationSession) async {
        guard let lyrics, let trackId = displayTrack?.id else { return }

        let mainTexts: [String]
        let bgTexts: [String]

        if let richsync = lyrics.richsync {
            let lines = richsync.sections.flatMap(\.lines)
            mainTexts = lines.map(\.text)
            bgTexts = lines.map { $0.bgVox?.text ?? "" }
        } else {
            mainTexts = lyrics.lines.lines.map(\.text)
            bgTexts = []
        }

        let allTexts = mainTexts + bgTexts
        let target = selectedTranslationLanguage ?? "auto"

        if let cached = TranslationCache.get(trackId: trackId, texts: allTexts, target: target, method: "device") {
            applyTranslations(cached, mainCount: mainTexts.count, bgCount: bgTexts.count)
            return
        }

        translationService.isTranslating = true

        do {
            let allTranslated = try await translationService.translateBatch(
                allTexts, session: session)
            applyTranslations(allTranslated, mainCount: mainTexts.count, bgCount: bgTexts.count)
            TranslationCache.set(allTranslated, trackId: trackId, texts: allTexts, target: target, method: "device")
        } catch {
            print("[Translation] failed: \(error)")
        }
        translationService.isTranslating = false
    }

    private func applyTranslations(_ all: [String], mainCount: Int, bgCount: Int) {
        translations = Array(all.prefix(mainCount))
        if bgCount > 0 {
            bgVoxTranslations = Array(all.suffix(bgCount))
        }
    }

    private func loadLyrics() {
        guard let track = displayTrack else {
            print("[Lyrics] loadLyrics called but no current track")
            return
        }
        print(
            "[Lyrics] fetching for '\(track.name)' by '\(track.displayArtist)' on '\(track.albumName)'"
        )
        lyrics = nil
        translations = nil
        bgVoxTranslations = nil
        Task {
            do {
                let jlf = try await umiClient.fetchLyrics(
                    track: track.name,
                    artist: track.displayArtist,
                    album: track.albumName
                )
                print(
                    "[Lyrics] success, \(jlf.lines.lines.count) lines from source '\(jlf.source)'")
                await MainActor.run { self.lyrics = jlf }
            } catch {
                print("[Lyrics] fetch failed: \(error)")
                await MainActor.run { self.lyrics = nil }
            }
        }
    }
}

#Preview {
    NowPlayingSheetView(player: .preview, remote: .previewLocalActive, dismiss: {})
}

#Preview("Remote") {
    NowPlayingSheetView(player: .previewIdle, remote: .previewRemote, dismiss: {})
}

// MARK: - Queue sheet

private struct NowPlayingQueueSheet: View {
    let tracks: [Track]
    let currentTrackId: Int?
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        NavigationStack {
            List(Array(tracks.enumerated()), id: \.element.id) { _, track in
                HStack(spacing: 12) {
                    ArtworkImage(url: track.artUrl, size: 44, cornerRadius: 6)

                    VStack(alignment: .leading, spacing: 2) {
                        Text(track.name)
                            .font(.subheadline)
                            .lineLimit(1)
                            .foregroundStyle(
                                track.id == currentTrackId ? Color.accentColor : .primary)
                        Text(track.displayArtist)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            .lineLimit(1)
                    }

                    Spacer()

                    if track.id == currentTrackId {
                        Image(systemName: "waveform")
                            .foregroundStyle(.tint)
                            .font(.caption)
                    }
                }
                .padding(.vertical, 2)
            }
            .navigationTitle("Queue")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Done") { dismiss() }
                }
            }
        }
    }
}
