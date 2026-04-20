import SwiftUI

/// Full-screen background: blurred spiral-bloom layers of the artwork, fading to black downward.
struct SpiralBackground: View {
    let url: String?

    var body: some View {
        GeometryReader { geo in
            ZStack(alignment: .top) {
                Color.black

                if let urlString = url, let imageURL = URL(string: urlString) {
                    // Base fill — large, centered near top, heavy blur
                    artLayer(imageURL, scale: 1.6, geo: geo, blur: 70, rotation: 0, opacity: 0.75)

                    // Spiral layers for texture
                    artLayer(imageURL, scale: 1.9, geo: geo, blur: 45, rotation: 30, opacity: 0.25, blend: .screen)
                    artLayer(imageURL, scale: 1.4, geo: geo, blur: 28, rotation: -20, opacity: 0.20, blend: .screen)
                    artLayer(imageURL, scale: 1.2, geo: geo, blur: 12, rotation: 12, opacity: 0.18, blend: .softLight)
                }

                // Fade to black: transparent at top, solid black by ~70%
                LinearGradient(
                    stops: [
                        .init(color: .clear, location: 0.0),
                        .init(color: .black.opacity(0.15), location: 0.30),
                        .init(color: .black.opacity(0.70), location: 0.55),
                        .init(color: .black, location: 0.72),
                    ],
                    startPoint: .top,
                    endPoint: .bottom
                )
            }
            .frame(width: geo.size.width, height: geo.size.height)
        }
        .ignoresSafeArea()
    }

    @ViewBuilder
    private func artLayer(
        _ url: URL,
        scale: CGFloat,
        geo: GeometryProxy,
        blur: CGFloat,
        rotation: Double,
        opacity: Double,
        blend: BlendMode = .normal
    ) -> some View {
        let side = geo.size.width * scale
        AsyncImage(url: url) { phase in
            switch phase {
            case .success(let image):
                image.resizable().aspectRatio(contentMode: .fill)
            default:
                Color.clear
            }
        }
        .frame(width: side, height: side)
        .clipped()
        .blur(radius: blur)
        .rotationEffect(.degrees(rotation))
        .opacity(opacity)
        .blendMode(blend)
        .frame(width: geo.size.width, alignment: .center)
        .allowsHitTesting(false)
    }
}

#Preview {
    ZStack {
        SpiralBackground(url: nil)
        Text("Album Title")
            .font(.largeTitle.bold())
            .foregroundStyle(.white)
    }
}
