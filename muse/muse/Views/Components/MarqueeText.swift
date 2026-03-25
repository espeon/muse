import SwiftUI

public struct MarqueeText: View {
    public var text: String
    public var font: UIFont
    public var leftFade: CGFloat
    public var rightFade: CGFloat
    public var startDelay: Double
    public var alignment: Alignment

    @State private var animate = false
    var isCompact = false

    public var body: some View {
        let stringWidth  = text.widthOfString(usingFont: font)
        let stringHeight = text.heightOfString(usingFont: font)

        let animation = Animation
            .linear(duration: Double(stringWidth) / 30)
            .delay(startDelay)
            .repeatForever(autoreverses: false)

        let nullAnimation = Animation.linear(duration: 0)

        GeometryReader { geo in
            let needsScrolling = stringWidth > geo.size.width

            ZStack {
                if needsScrolling {
                    makeMarqueeTexts(
                        stringWidth: stringWidth,
                        stringHeight: stringHeight,
                        animation: animation,
                        nullAnimation: nullAnimation
                    )
                    .frame(minWidth: 0, maxWidth: .infinity, minHeight: 0, maxHeight: .infinity, alignment: .topLeading)
                    .offset(x: leftFade)
                    .mask(fadeMask(leftFade: leftFade, rightFade: rightFade))
                    .frame(width: geo.size.width + leftFade)
                    .offset(x: -leftFade)
                } else {
                    Text(text)
                        .font(.init(font))
                        .onChange(of: text) { _, _ in animate = false }
                        .frame(minWidth: 0, maxWidth: .infinity, minHeight: 0, maxHeight: .infinity, alignment: alignment)
                }
            }
            .clipped()
            .onAppear { animate = needsScrolling }
            .onChange(of: text) { _, newValue in
                let newWidth = newValue.widthOfString(usingFont: font)
                if newWidth > geo.size.width {
                    animate = false
                    DispatchQueue.main.async { animate = true }
                } else {
                    animate = false
                }
            }
        }
        .frame(height: stringHeight)
        .frame(maxWidth: isCompact ? stringWidth : nil)
        .onDisappear { animate = false }
    }

    @ViewBuilder
    private func makeMarqueeTexts(
        stringWidth: CGFloat,
        stringHeight: CGFloat,
        animation: Animation,
        nullAnimation: Animation
    ) -> some View {
        Group {
            Text(text)
                .lineLimit(1)
                .font(.init(font))
                .offset(x: animate ? -stringWidth - stringHeight * 2 : 0)
                .animation(animate ? animation : nullAnimation, value: animate)
                .fixedSize(horizontal: true, vertical: false)

            Text(text)
                .lineLimit(1)
                .font(.init(font))
                .offset(x: animate ? 0 : stringWidth + stringHeight * 2)
                .animation(animate ? animation : nullAnimation, value: animate)
                .fixedSize(horizontal: true, vertical: false)
        }
    }

    @ViewBuilder
    private func fadeMask(leftFade: CGFloat, rightFade: CGFloat) -> some View {
        HStack(spacing: 0) {
            Rectangle().frame(width: 2).opacity(0)
            LinearGradient(colors: [.black.opacity(0), .black], startPoint: .leading, endPoint: .trailing)
                .frame(width: leftFade)
            LinearGradient(colors: [.black, .black], startPoint: .leading, endPoint: .trailing)
            LinearGradient(colors: [.black, .black.opacity(0)], startPoint: .leading, endPoint: .trailing)
                .frame(width: rightFade)
            Rectangle().frame(width: 2).opacity(0)
        }
    }

    public init(text: String, font: UIFont, leftFade: CGFloat, rightFade: CGFloat, startDelay: Double, alignment: Alignment? = nil) {
        self.text = text
        self.font = font
        self.leftFade = leftFade
        self.rightFade = rightFade
        self.startDelay = startDelay
        self.alignment = alignment ?? .topLeading
    }
}

extension MarqueeText {
    public func makeCompact(_ compact: Bool = true) -> Self {
        var view = self
        view.isCompact = compact
        return view
    }
}

extension String {
    func widthOfString(usingFont font: UIFont) -> CGFloat {
        self.size(withAttributes: [.font: font]).width
    }

    func heightOfString(usingFont font: UIFont) -> CGFloat {
        self.size(withAttributes: [.font: font]).height
    }
}
