import SwiftUI

struct SyncedLyricsView: View {
    let lyrics: JLF
    let currentTimeMs: Int
    let onSeek: ((Int) -> Void)?
    let fontDesign: Font.Design
    let fontWeight: Font.Weight

    @State private var currentLineIndex: Int = 0
    @Namespace private var scrollSpace

    init(
        lyrics: JLF, currentTimeMs: Int, onSeek: ((Int) -> Void)? = nil,
        fontDesign: Font.Design = .default, fontWeight: Font.Weight = .bold
    ) {
        self.lyrics = lyrics
        self.currentTimeMs = currentTimeMs
        self.onSeek = onSeek
        self.fontDesign = fontDesign
        self.fontWeight = fontWeight
    }

    private var lines: [SyncedLine] {
        lyrics.lines.lines
    }

    private func lineIndex(for timeMs: Int) -> Int {
        // Find the last line that starts before or at current time
        var index = 0
        for (i, line) in lines.enumerated() {
            if line.time <= timeMs {
                index = i
            } else {
                break
            }
        }
        return index
    }

    /// Calculate activation progress for a line (0 = inactive, 1 = active)
    /// Primes the next line starting 1.5s before it becomes active
    private func activationProgress(for index: Int) -> Double {
        let line = lines[index]
        let primeWindowMs = 1500

        if index == currentLineIndex {
            return 1.0
        } else if index == currentLineIndex + 1 {
            // Next line - check if we're within prime window
            let timeUntilActive = line.time - currentTimeMs
            if timeUntilActive > 0 && timeUntilActive <= primeWindowMs {
                // Ease-in curve for smoother ramp up
                let linearProgress = 1.0 - Double(timeUntilActive) / Double(primeWindowMs)
                let easedProgress = linearProgress * linearProgress  // quadratic ease-in
                return 0.6 * easedProgress
            }
        }
        return 0.0
    }

    var body: some View {
        ScrollViewReader { proxy in
            ScrollView(showsIndicators: false) {
                VStack(alignment: .leading, spacing: 24) {
                    // Top padding for scroll space
                    Spacer()
                        .frame(height: 100)

                    ForEach(Array(lines.enumerated()), id: \.element.id) { index, line in
                        LyricLineView(
                            line: line,
                            isActive: index == currentLineIndex,
                            activationProgress: activationProgress(for: index),
                            distanceFromActive: index - currentLineIndex,
                            onTap: onSeek != nil ? { onSeek?(line.time) } : nil,
                            fontDesign: fontDesign,
                            fontWeight: fontWeight
                        )
                        .id(line.id)
                    }

                    // Bottom padding
                    Spacer()
                        .frame(height: 200)
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.horizontal, 24)
            }
            .mask(
                VStack(spacing: 0) {
                    // Top fade
                    LinearGradient(
                        colors: [.clear, .black.opacity(0.1), .black.opacity(0.3), .black, .black],
                        startPoint: .top,
                        endPoint: .bottom
                    )
                    .frame(height: 200)

                    // Middle solid
                    Rectangle()
                        .fill(.black)

                    // Bottom fade
                    LinearGradient(
                        colors: [.black, .clear],
                        startPoint: .top,
                        endPoint: .bottom
                    )
                    .frame(height: 100)
                }
            )
            .onChange(of: currentTimeMs) { _, newTime in
                let newIndex = lineIndex(for: newTime)
                if newIndex != currentLineIndex {
                    currentLineIndex = newIndex
                    withAnimation(.easeInOut(duration: 0.3)) {
                        proxy.scrollTo(lines[newIndex].id, anchor: UnitPoint(x: 0.5, y: 0.4))
                    }
                }
            }
            .onAppear {
                currentLineIndex = lineIndex(for: currentTimeMs)
                proxy.scrollTo(lines[safe: currentLineIndex]?.id, anchor: UnitPoint(x: 0.5, y: 0.4))
            }
        }
    }
}

let BreakIndicatorSizeActive: CGFloat = 20
let BreakIndicatorSizeInactive: CGFloat = 15

let LineScaleActive: CGFloat = 1.0
let LineScaleInactive: CGFloat = 0.9

struct LyricLineView: View {
    let line: SyncedLine
    let isActive: Bool
    let activationProgress: Double  // 0 = inactive, 1 = fully active
    let distanceFromActive: Int
    let onTap: (() -> Void)?
    let fontDesign: Font.Design
    let fontWeight: Font.Weight

    private var normalizedDistance: Double {
        Double(abs(distanceFromActive))
    }

    // Lerp helper
    private func lerp(_ a: Double, _ b: Double, _ t: Double) -> Double {
        a + (b - a) * t
    }

    private var opacity: Double {
        let activeOpacity = 1.0
        let inactiveOpacity = max(0.15, exp(-normalizedDistance * 0.5))
        return lerp(inactiveOpacity, activeOpacity, activationProgress)
    }

    private var scale: Double {
        let activeScale = 1.0
        let inactiveScale = max(0.92, 1.0 - normalizedDistance * 0.02)
        return lerp(inactiveScale, activeScale, activationProgress)
    }

    private var blur: Double {
        let activeBlur = 0.0
        let inactiveBlur = min(normalizedDistance * 0.8, 3.0)
        return lerp(inactiveBlur, activeBlur, activationProgress)
    }

    private var yOffset: Double {
        if isActive { return 0 }
        let direction = distanceFromActive > 0 ? 1.0 : -1.0
        let inactiveOffset = direction * min(normalizedDistance * 2, 8)
        return lerp(inactiveOffset, 0, activationProgress)
    }

    var body: some View {
        Group {
            if line.text.isEmpty {
                // Instrumental break indicator
                HStack(spacing: isActive ? BreakIndicatorSizeActive : BreakIndicatorSizeInactive) {
                    ForEach(0..<3, id: \.self) { index in
                        Circle()
                            .fill(.white)
                            .frame(
                                width: isActive
                                    ? BreakIndicatorSizeActive : BreakIndicatorSizeInactive,
                                height: isActive
                                    ? BreakIndicatorSizeActive : BreakIndicatorSizeInactive
                            )
                            .scaleEffect(isActive ? 1.0 : 0.7)
                            .animation(
                                .spring(response: 0.4, dampingFraction: 0.6)
                                    .delay(Double(index) * 0.05),
                                value: isActive
                            )
                    }
                    Spacer()
                }
                .padding(.vertical, 16)
                .animation(.spring(response: 0.4, dampingFraction: 0.6), value: isActive)
            } else {
                Text(line.text)
                    .font(.system(size: 36, weight: fontWeight, design: fontDesign))
                    .multilineTextAlignment(.leading)
                    .fixedSize(horizontal: false, vertical: true)
            }
        }
        .foregroundColor(.white)
        .opacity(opacity)
        .scaleEffect(scale, anchor: .leading)
        .blur(radius: blur)
        .offset(y: yOffset)
        .animation(.spring(response: 0.35, dampingFraction: 0.8), value: isActive)
        .animation(.spring(response: 0.35, dampingFraction: 0.8), value: distanceFromActive)
        .animation(.linear(duration: 0.1), value: activationProgress)
        .contentShape(Rectangle())
        .onTapGesture {
            onTap?()
        }
    }
}

// Safe array subscript
extension Array {
    subscript(safe index: Index) -> Element? {
        indices.contains(index) ? self[index] : nil
    }
}

#Preview {
    let sampleLyrics = JLF(
        lines: SyncedLines(
            lines: [
                SyncedLine(time: 0, text: "First line of the song"),
                SyncedLine(time: 3000, text: "Second line comes here"),
                SyncedLine(time: 6000, text: "Third line with more words"),
                SyncedLine(time: 9000, text: ""),  // instrumental
                SyncedLine(time: 12000, text: "Back to singing now"),
                SyncedLine(time: 15000, text: "Another line goes here"),
                SyncedLine(time: 18000, text: "Keep on singing along"),
                SyncedLine(time: 21000, text: "The melody continues"),
                SyncedLine(time: 24000, text: "Almost at the end"),
                SyncedLine(time: 27000, text: "Final line of the song"),
            ],
            linesEnd: 30000
        ),
        source: "preview"
    )

    return ZStack {
        Color.black.ignoresSafeArea()
        SyncedLyricsView(
            lyrics: sampleLyrics, currentTimeMs: 6500, fontDesign: .default, fontWeight: .bold)
    }
}
