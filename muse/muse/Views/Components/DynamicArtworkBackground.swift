import SwiftUI

struct DynamicArtworkBackground: View {
    let url: URL?
    let move: Bool?

    @Environment(\.tabViewBottomAccessoryPlacement)
    private var placement

    @State private var start = Date()
    @State private var currentImage: UIImage?
    @State private var nextImage: UIImage?
    @State private var currentURL: URL?
    @State private var fadeProgress: CGFloat = 0.0

    var body: some View {
        GeometryReader { geometry in
            ZStack {
                Color.black

                Group {
                    if let current = currentImage {
                        backgroundLayer(image: Image(uiImage: current), geometry: geometry)
                            .opacity(1.0 - fadeProgress)
                    }
                    
                    if let next = nextImage {
                        backgroundLayer(image: Image(uiImage: next), geometry: geometry)
                            .opacity(fadeProgress)
                    }
                }.opacity(0.8)

            }
            .ignoresSafeArea()
        }
        .task(id: url) {
            guard url != currentURL else { return }

            if let newImage = await ImageCache.shared.image(for: url) {
                currentURL = url
                nextImage = newImage

                withAnimation(.easeInOut(duration: 0.5)) {
                    fadeProgress = 1.0
                }

                try? await Task.sleep(for: .milliseconds(500))

                if url == currentURL {
                    currentImage = newImage
                    nextImage = nil
                    fadeProgress = 0.0
                }
            }
        }
        .onAppear {
            Task {
                currentURL = url
                currentImage = await ImageCache.shared.image(for: url)
            }
        }
    }

    @ViewBuilder
    private func backgroundLayer(image: Image, geometry: GeometryProxy) -> some View {
        TimelineView(.animation) { timeline in
            let time = (move ?? true) ? timeline.date.timeIntervalSince(start) : 0

            image
                .resizable()
                .aspectRatio(contentMode: .fill)
                .frame(width: geometry.size.width, height: geometry.size.height)
                .layerEffect(
                    ShaderLibrary.dreamySpiral(
                        .float(time),
                        .float2(geometry.size)
                    ),
                    maxSampleOffset: .init(width: 150, height: 150)
                )
                .blur(radius: UIDevice.current.userInterfaceIdiom == .pad ? 84 : 48)
                .scaleEffect(0.85)
        }
    }
}

#Preview {
    DynamicArtworkBackground(url: URL(string: "https://kagi.com/proxy/Paramore_-_Brand_New_Eyes.png?c=9cn5Kxse4yD05EJkf6QML9dK4clUbdQ9Oq4d5gDoyHDU2DEo8Di3JZqdJB9gnlKCBlaC6xWejHGPYBR6n3zUPKct_3D1OQmkwKnUj7IuB6vkkl8FJpUEZwr1jpPPHccn"), move: true)
}
