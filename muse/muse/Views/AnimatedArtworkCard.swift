import SwiftUI

enum TransitionDirection {
    case forward
    case backward
}

struct AnimatedArtworkCard: View {
    let url: URL?
    let queueItemId: String?
    let queueIndex: Int?
    let size: CGFloat
    let cornerRadius: CGFloat
    let playingScale: CGFloat

    @State private var currentURL: URL?
    @State private var previousURL: URL?
    @State private var currentItemId: String?
    @State private var previousIndex: Int?
    @State private var showCurrent = true
    @State private var direction: TransitionDirection = .forward
    @State private var hasAppeared = false

    init(
        url: URL?, queueItemId: String?, queueIndex: Int?, size: CGFloat, cornerRadius: CGFloat,
        playingScale: CGFloat
    ) {
        self.url = url
        self.queueItemId = queueItemId
        self.queueIndex = queueIndex
        self.size = size
        self.cornerRadius = cornerRadius
        self.playingScale = playingScale
        _currentURL = State(initialValue: url)
        _currentItemId = State(initialValue: queueItemId)
        _previousIndex = State(initialValue: queueIndex)
    }

    var body: some View {
        ZStack {
            // Current card (slides in from underneath)
            if let currentURL = currentURL {
                AsyncImage(url: currentURL) { image in
                    image.resizable().aspectRatio(contentMode: .fill)
                } placeholder: {
                    Color.gray
                }
                .frame(width: size, height: size)
                .cornerRadius(cornerRadius)
                .shadow(color: .black.opacity(0.4), radius: 30, x: 0, y: 15)
                .scaleEffect(
                    playingScale
                )
                .offset(
                    x: showCurrent ? 0 : (direction == .forward ? -400 : 400),
                    y: showCurrent ? 0 : -10
                )
                .transition(.identity)
                .id(currentURL.absoluteString)
                .zIndex(0)
            }

            // Previous card (slides out on top)
            if let previousURL = previousURL {
                AsyncImage(url: previousURL) { image in
                    image.resizable().aspectRatio(contentMode: .fill)
                } placeholder: {
                    Color.gray
                }
                .frame(width: size, height: size)
                .cornerRadius(cornerRadius)
                .shadow(color: .black.opacity(0.4), radius: 30, x: 0, y: 15)
                .scaleEffect(
                    playingScale
                )
                .offset(
                    x: showCurrent ? (direction == .forward ? 400 : -400) : 0,
                    y: showCurrent ? -10 : 0
                )
                .transition(.identity)
                .id(previousURL.absoluteString)
                .zIndex(1)
            }
        }
        .transaction { transaction in
            if !hasAppeared {
                transaction.animation = nil
                transaction.disablesAnimations = true
            }
        }
        .animation(
            hasAppeared ? .spring(response: 0.6, dampingFraction: 0.75) : nil, value: showCurrent
        )
        .animation(.spring(response: 0.5, dampingFraction: 0.7), value: playingScale)
        .onChange(of: queueIndex) { oldIdx, newIdx in
            guard hasAppeared else { return }
            guard let newIdx = newIdx, newIdx != previousIndex else { return }

            // Detect direction based on queue index change
            if let oldIdx = oldIdx {
                direction = newIdx > oldIdx ? .forward : .backward
            } else {
                direction = .forward
            }

            previousURL = currentURL
            previousIndex = newIdx
            currentItemId = queueItemId
            showCurrent = false

            // Trigger transition immediately
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.05) {
                currentURL = url
                showCurrent = true

                // Clean up previous after animation
                DispatchQueue.main.asyncAfter(deadline: .now() + 0.7) {
                    previousURL = nil
                }
            }
        }
        .onAppear {
            // Enable animations after sheet fully opens (delay for sheet animation)
            DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
                hasAppeared = true
            }
        }
    }
}
