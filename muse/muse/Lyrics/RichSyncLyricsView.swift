import SwiftUI

let ScrollAnchor = UnitPoint(x: 0.5, y: 0.3)

extension String {
    var isCJK: Bool {
        for scalar in unicodeScalars {
            let code = scalar.value
            if (0x4E00...0x9FFF).contains(code) || // CJK Unified Ideographs
               (0x3400...0x4DBF).contains(code) || // CJK Extension A
               (0x3040...0x309F).contains(code) || // Hiragana
               (0x30A0...0x30FF).contains(code) || // Katakana
               (0xAC00...0xD7AF).contains(code) {  // Hangul Syllables
                return true
            }
        }
        return false
    }
}

struct RenderedWord: Identifiable, Equatable {
    let id: Int
    let text: String
    let segment: SyncedRichLineSegment
    let offsetInSegment: Int
    let totalInSegment: Int
    let isCJK: Bool

    static func == (lhs: RenderedWord, rhs: RenderedWord) -> Bool {
        lhs.id == rhs.id &&
            lhs.text == rhs.text &&
            lhs.segment.timeStart == rhs.segment.timeStart
    }
}

struct RichLyricsView: View {
    let richsync: SyncedRich
    let currentTimeMs: Int
    let onSeek: ((Int) -> Void)?
    let fontDesign: Font.Design
    let fontWeight: Font.Weight
    let fontSizeMultiplier: Double
    let fadeCompletedLines: Bool
    let translations: [String]?
    let bgVoxTranslations: [String]?

    @State private var currentLineIndex: Int = 0
    @State private var isScrollViewGestureScroll: Bool = false
    @State private var scrollValue: CGFloat = 0
    @Namespace private var scrollSpace

    private var allLines: [(sectionIndex: Int, lineIndex: Int, line: SyncedRichLine)] {
        richsync.sections.enumerated().flatMap { sectionIndex, section in
            section.lines.enumerated().map { lineIndex, line in
                (sectionIndex, lineIndex, line)
            }
        }
    }

    private func lineIndex(for timeMs: Int) -> Int {
        let lines = allLines
        var lo = 0
        var hi = lines.count
        while lo < hi {
            let mid = lo + (hi - lo) / 2
            if lines[mid].line.timeStart <= timeMs {
                lo = mid + 1
            } else {
                hi = mid
            }
        }
        return max(0, lo - 1)
    }

    private func activationProgress(for index: Int) -> Double {
        let primeWindowMs = 1500

        if index == currentLineIndex {
            return 1.0
        } else if index == currentLineIndex + 1 {
            let line = allLines[index].line
            let timeUntilActive = line.timeStart - currentTimeMs
            if timeUntilActive > 0 && timeUntilActive <= primeWindowMs {
                let linearProgress = 1.0 - Double(timeUntilActive) / Double(primeWindowMs)
                return 0.9 * linearProgress * linearProgress
            }
        }
        return 0.0
    }

    var body: some View {
        GeometryReader { geometry in
            let baseWidth: CGFloat = 450
            let widthMultiplier = min(max(Double(geometry.size.width / baseWidth), 0.75), 1.4)
            let effectiveMultiplier = widthMultiplier * fontSizeMultiplier

            ScrollViewReader { proxy in
                ScrollView(showsIndicators: false) {
                LazyVStack(alignment: .leading, spacing: 24) {
                    Spacer().frame(height: 100)

                    ForEach(0..<allLines.count, id: \.self) { index in
                        let item = allLines[index]
                        let hasBgVox = item.line.bgVox != nil
                        let bgVoxActive = hasBgVox && item.line.bgVox.map { currentTimeMs >= $0.timeStart && currentTimeMs < $0.timeEnd } ?? false
                        let shouldUpdate = index == currentLineIndex || bgVoxActive

                        RichLyricLineView(
                            line: item.line,
                            translation: translations?[safe: index],
                            bgVoxTranslation: bgVoxTranslations?[safe: index],
                            isActive: index == currentLineIndex || bgVoxActive,
                            activationProgress: activationProgress(for: index),
                            distanceFromActive: index - currentLineIndex,
                            currentTimeMs: shouldUpdate ? currentTimeMs : 0,
                            onTap: onSeek != nil ? { onSeek?(item.line.timeStart) } : nil,
                            fontDesign: fontDesign,
                            fontWeight: fontWeight,
                            fontSizeMultiplier: effectiveMultiplier,
                            fadeCompletedLines: fadeCompletedLines
                        )
                        .id(index)
                    }
                    
                    Spacer().frame(height: geometry.size.height)
                }
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.horizontal, 24)
                .offset(y: scrollValue)
            }
            .mask(
                VStack(spacing: 0) {
                    LinearGradient(
                        colors: [.clear, .black.opacity(0.1), .black.opacity(0.3), .black, .black],
                        startPoint: .top,
                        endPoint: .bottom
                    )
                    .frame(height: 200)
                    
                    Rectangle().fill(.black)
                    
                    LinearGradient(
                        colors: [.black, .clear],
                        startPoint: .top,
                        endPoint: .bottom
                    )
                    .frame(height: 100)
                }
            )
            .gesture(
                DragGesture()
                    .onChanged { value in
                        isScrollViewGestureScroll = true
                        scrollValue = value.translation.height
                    }
                    .onEnded { _ in
                        handleScrollDragEnd(proxy: proxy)
                    }
            )
            .onChange(of: currentTimeMs) { _, newTime in
                let newIndex = lineIndex(for: newTime)
                if newIndex != currentLineIndex {
                    currentLineIndex = newIndex
                    handleNewHighlightLine(proxy: proxy)
                }
            }
            .onAppear {
                currentLineIndex = lineIndex(for: currentTimeMs)
                proxy.scrollTo(currentLineIndex, anchor: ScrollAnchor)
            }
            }
        }
    }
    

    private func handleScrollDragEnd(proxy: ScrollViewProxy) {
        DispatchQueue.main.asyncAfter(deadline: .now() + 1) {
            withAnimation(.easeInOut(duration: 0.3)) {
                scrollValue = 0
                isScrollViewGestureScroll = false
                if allLines.count >= 1 {
                    proxy.scrollTo(currentLineIndex, anchor: currentLineIndex > 0 ? ScrollAnchor : .top)
                } else {
                    proxy.scrollTo(0, anchor: .top)
                }
            }
        }
    }

    private func handleNewHighlightLine(proxy: ScrollViewProxy) {
        if allLines.count >= 1 && !isScrollViewGestureScroll {
            DispatchQueue.main.async {
                withAnimation(.easeInOut(duration: 0.3)) {
                    proxy.scrollTo(currentLineIndex, anchor: currentLineIndex > 0 ? ScrollAnchor : .top)
                }
            }
        }
    }
}

struct BgVoxLineView: View, Equatable {
    let bgVox: SyncedRichBackgroundLine
    let translation: String?
    let currentTimeMs: Int
    let horizontalAlignment: HorizontalAlignment
    let unitPointAnchor: UnitPoint
    let fontDesign: Font.Design
    let fontWeight: Font.Weight
    let fontSizeMultiplier: Double

    @State private var cachedWords: [RenderedWord] = []
    @State private var isInitialized: Bool = false

    static func == (lhs: BgVoxLineView, rhs: BgVoxLineView) -> Bool {
        let lhsActive = lhs.currentTimeMs >= lhs.bgVox.timeStart && lhs.currentTimeMs < lhs.bgVox.timeEnd
        let rhsActive = rhs.currentTimeMs >= rhs.bgVox.timeStart && rhs.currentTimeMs < rhs.bgVox.timeEnd

        if lhs.bgVox.text != rhs.bgVox.text || lhs.translation != rhs.translation {
            return false
        }
        if (lhsActive || rhsActive) && lhs.currentTimeMs != rhs.currentTimeMs {
            return false
        }
        return true
    }

    private var words: [RenderedWord] {
        cachedWords.isEmpty ? renderedWords(text: bgVox.text, segments: bgVox.segments) : cachedWords
    }

    private var isBgVoxActive: Bool {
        currentTimeMs >= bgVox.timeStart && currentTimeMs < bgVox.timeEnd
    }

    private var opacity: Double {
        isBgVoxActive ? 0.8 : 0.3
    }

    private var scale: Double {
        isBgVoxActive ? 1.0 : 0.95
    }

    private var blur: Double {
        isBgVoxActive ? 0.0 : 1.5
    }

    var body: some View {
        VStack(alignment: horizontalAlignment, spacing: 4) {
            TextWrapLayout(spacing: 0, alignment: horizontalAlignment) {
                ForEach(words) { word in
                    bgVoxWordView(for: word)
                }
            }
            .transaction { tx in
                tx.animation = nil
            }
            .frame(maxWidth: .infinity, alignment: Alignment(horizontal: horizontalAlignment, vertical: .center))
            .drawingGroup()
            .opacity(opacity)
            .scaleEffect(scale, anchor: unitPointAnchor)
            .blur(radius: blur)
            .animation(.spring(response: 0.35, dampingFraction: 0.8), value: isBgVoxActive)

            if let translation = translation, !translation.isEmpty {
                Text(translation)
                    .font(.system(size: 16 * fontSizeMultiplier, weight: .regular, design: fontDesign))
                    .foregroundColor(.white.opacity(0.5))
                    .lineLimit(nil)
                    .multilineTextAlignment(horizontalAlignment == .leading ? .leading : (horizontalAlignment == .center ? .center : .trailing))
                    .fixedSize(horizontal: false, vertical: true)
                    .frame(maxWidth: .infinity, alignment: Alignment(horizontal: horizontalAlignment, vertical: .center))
                    .opacity(opacity)
            }
        }
        .onAppear {
            if !isInitialized {
                cachedWords = renderedWords(text: bgVox.text, segments: bgVox.segments)
                isInitialized = true
            }
        }
    }

    @ViewBuilder
    private func bgVoxWordView(for word: RenderedWord) -> some View {
        let bgVoxComplete = currentTimeMs >= bgVox.timeEnd
        let segmentDuration = word.segment.timeEnd - word.segment.timeStart
        let isSegmentActive = currentTimeMs >= word.segment.timeStart && currentTimeMs < word.segment.timeEnd
        let isSegmentComplete = currentTimeMs >= word.segment.timeEnd

        Text(word.text)
            .font(.system(size: 28 * fontSizeMultiplier, weight: fontWeight, design: fontDesign))
            .foregroundColor(.white)
            .lineLimit(nil)
            .fixedSize(horizontal: false, vertical: true)
            .visualEffect { content, proxy in
                let progress: Float
                if bgVoxComplete {
                    progress = 1.0
                } else if isSegmentComplete {
                    progress = 1.0
                } else if isSegmentActive {
                    progress = Float(currentTimeMs - word.segment.timeStart) / Float(max(1, segmentDuration))
                } else {
                    progress = 0.0
                }

                let edgeWidth = 20.0 / Float(proxy.size.width)
                let gradientWidth = edgeWidth * 2.0
                let sweep = progress * (1.0 + 2.0 * gradientWidth) - gradientWidth

                return content
                    .colorEffect(
                        ShaderLibrary.karaokeSweep(
                            .float2(proxy.size),
                            .float(sweep),
                            .float(20)
                        )
                    )
            }
    }
}

struct RichLyricLineView: View, Equatable {
    let line: SyncedRichLine
    let translation: String?
    let bgVoxTranslation: String?
    let isActive: Bool
    let activationProgress: Double
    let distanceFromActive: Int
    let currentTimeMs: Int
    let onTap: (() -> Void)?
    let fontDesign: Font.Design
    let fontWeight: Font.Weight
    let fontSizeMultiplier: Double
    let fadeCompletedLines: Bool

    @State private var cachedWords: [RenderedWord] = []
    @State private var isInitialized: Bool = false

    static func == (lhs: RichLyricLineView, rhs: RichLyricLineView) -> Bool {
        if lhs.line.text != rhs.line.text || lhs.isActive != rhs.isActive || lhs.translation != rhs.translation || lhs.bgVoxTranslation != rhs.bgVoxTranslation {
            return false
        }

        let hasBgVox = lhs.line.bgVox != nil
        let lhsBgVoxActive = hasBgVox && lhs.line.bgVox.map {
            lhs.currentTimeMs >= $0.timeStart && lhs.currentTimeMs < $0.timeEnd
        } ?? false
        let rhsBgVoxActive = hasBgVox && rhs.line.bgVox.map {
            rhs.currentTimeMs >= $0.timeStart && rhs.currentTimeMs < $0.timeEnd
        } ?? false

        // compare currentTimeMs if main line is active OR bgVox is active (on either side)
        if (lhs.isActive || lhsBgVoxActive || rhsBgVoxActive) && lhs.currentTimeMs != rhs.currentTimeMs {
            return false
        }
        return true
    }

    private var words: [RenderedWord] {
        cachedWords.isEmpty ? renderedWords(from: line) : cachedWords
    }

    private func foregroundColor(for word: RenderedWord) -> Color {
        if word.text == " " {
            return .white
        }

        if isActive {
            let sweep = sweepProgress(for: word)
            let charPosition = Double(word.offsetInSegment + 1) / Double(word.totalInSegment)
            return charPosition <= sweep ? .white : .white.opacity(0.5)
        } else {
            return .white.opacity(0.3)
        }
    }

    private func sweepProgress(for word: RenderedWord) -> Double {
        guard isActive else { return 0.0 }

        let isSegmentActive =
            word.segment.timeStart <= currentTimeMs &&
            currentTimeMs < word.segment.timeEnd

        guard isSegmentActive else {
            return currentTimeMs >= word.segment.timeEnd ? 1.0 : 0.0
        }

        let segmentDuration = word.segment.timeEnd - word.segment.timeStart
        let elapsed = currentTimeMs - word.segment.timeStart
        return Double(elapsed) / Double(max(1, segmentDuration))
    }
    
    
    private var normalizedDistance: Double {
        Double(abs(distanceFromActive))
    }

    // Lerp helper
    private func lerp(_ a: Double, _ b: Double, _ t: Double) -> Double {
        a + (b - a) * t
    }

    private var opacity: Double {
        if isActive {
            return 1.0
        }
        // Fade completed lines if enabled (Apple Music style)
        if fadeCompletedLines && distanceFromActive < 0 {
            return 0.0
        }
        return 0.5
    }

    private var scale: Double {
        let activeScale = 1.0
        let inactiveScale = max(0.92, 1.0 - normalizedDistance * 0.02)
        return lerp(inactiveScale, activeScale, activationProgress)
    }

    private var blur: Double {
        let activeBlur = 0.0
        let inactiveBlur = min(normalizedDistance * 0.4, 1.5)
        return lerp(inactiveBlur, activeBlur, activationProgress)
    }

    private var shadowRadius: Double {
        let activeShadow = 12.0
        let inactiveShadow = 0.0
        return lerp(inactiveShadow, activeShadow, activationProgress)
    }

    private var yOffset: Double {
        if isActive { return 0 }
        let direction = distanceFromActive > 0 ? 1.0 : -1.0
        let inactiveOffset = direction * min(normalizedDistance * 2, 8)
        return lerp(inactiveOffset, 0, activationProgress)
    }

    private var horizontalAlignment: HorizontalAlignment {
        if line.agent == "v0" { return .leading }
        if line.agent == "v1" { return .leading }
        if line.agent == "v1000" { return .center }
        return .trailing
    }

    private var textAlignment: TextAlignment {
        if line.agent == "v0" { return .leading }
        if line.agent == "v1" { return .leading }
        if line.agent == "v1000" { return .center }
        return .trailing
    }

    private var unitPointAnchor: UnitPoint {
        if line.agent == "v0" { return .leading }
        if line.agent == "v1" { return .leading }
        if line.agent == "v1000" { return .center }
        return .trailing
    }

    var body: some View {
        VStack(alignment: horizontalAlignment, spacing: 4) {
            VStack(alignment: horizontalAlignment, spacing: 4) {
                TextWrapLayout(spacing: 0, alignment: horizontalAlignment) {
                    ForEach(words) { word in
                        wordView(for: word)
                    }
                }
                .transaction { tx in
                    tx.animation = nil
                }
                .frame(maxWidth: .infinity, alignment: Alignment(horizontal: horizontalAlignment, vertical: .center))
                .drawingGroup()
                .blur(radius: blur)
                .scaleEffect(scale, anchor: unitPointAnchor)
                .shadow(color: .black.opacity(0.3), radius: shadowRadius, x: 0, y: 0)
                .opacity(opacity)
                .animation(.spring(response: 0.35, dampingFraction: 0.8), value: isActive)
                .animation(.spring(response: 0.35, dampingFraction: 0.8), value: distanceFromActive)

                if let translation = translation {
                    Text(translation)
                        .font(.system(size: 20 * fontSizeMultiplier, weight: .regular, design: fontDesign))
                        .foregroundColor(.white.opacity(0.6))
                        .lineLimit(nil)
                        .multilineTextAlignment(textAlignment)
                        .fixedSize(horizontal: false, vertical: true)
                        .blur(radius: blur)
                        .opacity(opacity)
                        .frame(maxWidth: .infinity, alignment: Alignment(horizontal: horizontalAlignment, vertical: .center))
                }
            }

            if let bgVox = line.bgVox {
                BgVoxLineView(
                    bgVox: bgVox,
                    translation: bgVoxTranslation,
                    currentTimeMs: currentTimeMs,
                    horizontalAlignment: horizontalAlignment,
                    unitPointAnchor: unitPointAnchor,
                    fontDesign: fontDesign,
                    fontWeight: fontWeight,
                    fontSizeMultiplier: fontSizeMultiplier
                )
            }
        }
        .frame(maxWidth: .infinity, alignment: Alignment(horizontal: horizontalAlignment, vertical: .center))
        .offset(y: yOffset)
        .contentShape(Rectangle())
        .onTapGesture {
            onTap?()
        }
        .onAppear {
            if !isInitialized {
                cachedWords = renderedWords(from: line)
                isInitialized = true
            }
        }
    }

    @ViewBuilder
    private func wordView(for word: RenderedWord) -> some View {
        if isActive {
            let segmentDuration = word.segment.timeEnd - word.segment.timeStart
            let isLongSegment = segmentDuration >= 2000
            let isSegmentActive = currentTimeMs >= word.segment.timeStart && currentTimeMs < word.segment.timeEnd

            Text(word.text)
                .font(.system(size: 36 * fontSizeMultiplier, weight: fontWeight, design: fontDesign))
                .foregroundColor(.white)
                .lineLimit(nil)
                .fixedSize(horizontal: false, vertical: true)
                .visualEffect { content, proxy in
                    let progress = isSegmentActive
                        ? Float(currentTimeMs - word.segment.timeStart) / Float(max(1, segmentDuration))
                        : (currentTimeMs >= word.segment.timeEnd ? 1.0 : 0.0)

                    let edgeWidth = 20.0 / Float(proxy.size.width)
                    let gradientWidth = edgeWidth * 2.0
                    let sweep = progress * (1.0 + 2.0 * gradientWidth) - gradientWidth

                    let waveAmp: Float = isLongSegment && isSegmentActive ? 4.0 * (0.5 - abs(progress - 0.5)) : 0.0

                    return content
                        .colorEffect(
                            ShaderLibrary.karaokeSweep(
                                .float2(proxy.size),
                                .float(sweep),
                                .float(20)
                            )
                        )
                        .distortionEffect(
                            ShaderLibrary.karaokeWave(
                                .float2(proxy.size),
                                .float(progress),
                                .float(waveAmp)
                            ),
                            maxSampleOffset: .zero
                        )
                }
        } else {
            Text(word.text)
                .font(.system(size: 36 * fontSizeMultiplier, weight: fontWeight, design: fontDesign))
                .foregroundColor(.white)
                .lineLimit(nil)
                .fixedSize(horizontal: false, vertical: true)
        }
    }
}

func renderedWords(
    text: String,
    segments: [SyncedRichLineSegment]
) -> [RenderedWord] {
    var result: [RenderedWord] = []
    var cursor = text.startIndex

    let lineIsCJK = text.isCJK

    if lineIsCJK {
        for segment in segments {
            let searchText = segment.text
            let searchRange = cursor..<text.endIndex

            guard let range = text.range(
                of: searchText,
                options: [],
                range: searchRange
            ) else {
                continue
            }

            let end = range.upperBound
            let segmentText = String(text[range])

            for (charIndex, char) in segmentText.enumerated() {
                let wordId = result.count
                result.append(RenderedWord(
                    id: wordId,
                    text: String(char),
                    segment: segment,
                    offsetInSegment: charIndex,
                    totalInSegment: segmentText.count,
                    isCJK: true
                ))
            }

            let hasTrailingSpace =
                end < text.endIndex &&
                text[end] == " "

            if hasTrailingSpace {
                let wordId = result.count
                result.append(RenderedWord(
                    id: wordId,
                    text: " ",
                    segment: segment,
                    offsetInSegment: 0,
                    totalInSegment: 1,
                    isCJK: true
                ))
                cursor = text.index(after: end)
            } else {
                cursor = end
            }
        }
    } else {
        var segmentIndex = 0
        while segmentIndex < segments.count {
            var wordSegments: [SyncedRichLineSegment] = []
            var wordText = ""

            while segmentIndex < segments.count {
                let segment = segments[segmentIndex]
                let searchRange = cursor..<text.endIndex

                guard let range = text.range(
                    of: segment.text,
                    options: [],
                    range: searchRange
                ) else {
                    segmentIndex += 1
                    continue
                }

                let end = range.upperBound
                let segmentText = String(text[range])

                wordSegments.append(segment)
                wordText += segmentText
                cursor = end

                let hasTrailingSpace = end < text.endIndex && text[end] == " "

                if hasTrailingSpace {
                    cursor = text.index(after: end)
                    segmentIndex += 1
                    break
                }

                segmentIndex += 1

                if segmentIndex >= segments.count {
                    break
                }
            }

            if !wordText.isEmpty {
                let wordId = result.count
                result.append(RenderedWord(
                    id: wordId,
                    text: wordText,
                    segment: wordSegments[0],
                    offsetInSegment: 0,
                    totalInSegment: 1,
                    isCJK: false
                ))

                if cursor < text.endIndex && text[text.index(before: cursor)] == " " {
                    let spaceId = result.count
                    result.append(RenderedWord(
                        id: spaceId,
                        text: " ",
                        segment: wordSegments.last ?? wordSegments[0],
                        offsetInSegment: 0,
                        totalInSegment: 1,
                        isCJK: false
                    ))
                }
            }
        }
    }

    return result
}

func renderedWords(
    from line: SyncedRichLine
) -> [RenderedWord] {
    renderedWords(text: line.text, segments: line.segments)
}

struct TextWrapLayout: Layout {
    var spacing: CGFloat = 0
    var alignment: HorizontalAlignment = .leading

    struct LineInfo {
        var width: CGFloat
        var height: CGFloat
        var subviews: [(index: Int, size: CGSize)]
    }

    struct LayoutCache {
        var lines: [LineInfo]
        var totalHeight: CGFloat
    }

    func makeCache(subviews: Subviews) -> LayoutCache? {
        nil
    }

    private func computeLines(
        subviews: Subviews,
        maxWidth: CGFloat
    ) -> (lines: [LineInfo], totalHeight: CGFloat) {
        var lines: [LineInfo] = []
        var currentLine = LineInfo(width: 0, height: 0, subviews: [])

        for (index, subview) in subviews.enumerated() {
            let unconstrained = subview.sizeThatFits(.unspecified)
            let constrained = subview.sizeThatFits(
                ProposedViewSize(width: maxWidth, height: nil)
            )
            let size = unconstrained.width > maxWidth ? constrained : unconstrained

            if currentLine.width + size.width > maxWidth && !currentLine.subviews.isEmpty {
                lines.append(currentLine)
                currentLine = LineInfo(width: 0, height: 0, subviews: [])
            }

            currentLine.subviews.append((index, size))
            currentLine.width += size.width + (currentLine.subviews.count > 1 ? spacing : 0)
            currentLine.height = max(currentLine.height, size.height)
        }

        if !currentLine.subviews.isEmpty {
            lines.append(currentLine)
        }

        let totalHeight = lines.reduce(0) { $0 + $1.height }
        return (lines, totalHeight)
    }

    func sizeThatFits(
        proposal: ProposedViewSize,
        subviews: Subviews,
        cache: inout LayoutCache?
    ) -> CGSize {
        let maxWidth = proposal.width ?? .infinity
        if cache == nil {
            let result = computeLines(subviews: subviews, maxWidth: maxWidth)
            cache = LayoutCache(lines: result.lines, totalHeight: result.totalHeight)
        }
        return CGSize(width: maxWidth, height: cache!.totalHeight)
    }

    func placeSubviews(
        in bounds: CGRect,
        proposal: ProposedViewSize,
        subviews: Subviews,
        cache: inout LayoutCache?
    ) {
        let maxWidth = proposal.width ?? bounds.width
        if cache == nil {
            let result = computeLines(subviews: subviews, maxWidth: maxWidth)
            cache = LayoutCache(lines: result.lines, totalHeight: result.totalHeight)
        }

        let lines = cache!.lines

        var indicesToSkip = Set<Int>()
        for (lineIndex, line) in lines.enumerated() {
            if lineIndex > 0, let firstItem = line.subviews.first {
                if firstItem.size.width > 0 && firstItem.size.width < 10 && firstItem.size.height > 20 {
                    indicesToSkip.insert(firstItem.index)
                }
            }
        }

        var y = bounds.minY
        for line in lines {
            var adjustedWidth = line.width
            var adjustedSubviews = line.subviews

            if let firstIndex = adjustedSubviews.first?.index, indicesToSkip.contains(firstIndex) {
                let firstSize = adjustedSubviews.first!.size
                adjustedWidth -= firstSize.width
                if adjustedSubviews.count > 1 {
                    adjustedWidth -= spacing
                }
                adjustedSubviews.removeFirst()
            }

            let xOffset: CGFloat
            switch alignment {
            case .leading:
                xOffset = bounds.minX
            case .center:
                xOffset = bounds.minX + (bounds.width - adjustedWidth) / 2
            case .trailing:
                xOffset = bounds.maxX - adjustedWidth
            default:
                xOffset = bounds.minX
            }

            var x = xOffset
            for (index, size) in adjustedSubviews {
                subviews[index].place(
                    at: CGPoint(x: x, y: y),
                    proposal: ProposedViewSize(
                        width: min(size.width, bounds.width),
                        height: size.height
                    )
                )
                x += size.width + spacing
            }
            y += line.height
        }
    }
}

struct RichLyricsPreviewContainer: View {
    let richsync: SyncedRich
    let translations: [String]?
    let bgVoxTranslations: [String]?

    @State private var currentTimeMs: Int = 0
    @State private var isPlaying: Bool = false
    @State private var timer: Timer?

    var body: some View {
        ZStack(alignment: .bottom) {
            RichLyricsView(
                richsync: richsync,
                currentTimeMs: currentTimeMs,
                onSeek: { timeMs in
                    currentTimeMs = timeMs
                },
                fontDesign: .rounded,
                fontWeight: .semibold,
                fontSizeMultiplier: 1.0,
                fadeCompletedLines: false,
                translations: translations,
                bgVoxTranslations: bgVoxTranslations
            )

            VStack(spacing: 12) {
                HStack {
                    Text(formatTime(currentTimeMs))
                        .font(.system(size: 12, design: .monospaced))
                        .foregroundColor(.white)

                    Slider(
                        value: Binding(
                            get: { Double(currentTimeMs) },
                            set: { currentTimeMs = Int($0) }
                        ),
                        in: 0...Double(richsync.totalTime)
                    )
                    .tint(.white)

                    Text(formatTime(richsync.totalTime))
                        .font(.system(size: 12, design: .monospaced))
                        .foregroundColor(.white)
                }

                Button(action: togglePlayPause) {
                    Image(systemName: isPlaying ? "pause.circle.fill" : "play.circle.fill")
                        .font(.system(size: 44))
                        .foregroundColor(.white)
                }
            }
            .padding(.horizontal, 24)
            .padding(.bottom, 20)
            .background(
                LinearGradient(
                    colors: [.clear, .black.opacity(0.7)],
                    startPoint: .top,
                    endPoint: .bottom
                )
                .frame(height: 150)
            )
        }
        .background(.black)
        .onDisappear {
            timer?.invalidate()
        }
    }

    private func togglePlayPause() {
        isPlaying.toggle()

        if isPlaying {
            timer = Timer.scheduledTimer(withTimeInterval: 0.016, repeats: true) { _ in
                currentTimeMs += 16
                if currentTimeMs >= richsync.totalTime {
                    currentTimeMs = 0
                }
            }
        } else {
            timer?.invalidate()
            timer = nil
        }
    }

    private func formatTime(_ ms: Int) -> String {
        let totalSeconds = ms / 1000
        let minutes = totalSeconds / 60
        let seconds = totalSeconds % 60
        let milliseconds = (ms % 1000) / 10
        return String(format: "%d:%02d.%02d", minutes, seconds, milliseconds)
    }
}
