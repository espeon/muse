//
//  ArtworkImage.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import SwiftUI

struct ArtworkImage: View {
    let url: String?
    let size: CGFloat
    var cornerRadius: CGFloat = 8

    var body: some View {
        Group {
            if let urlString = url, let imageURL = URL(string: urlString) {
                AsyncImage(url: imageURL) { phase in
                    switch phase {
                    case .empty:
                        placeholder
                    case .success(let image):
                        image
                            .resizable()
                            .aspectRatio(contentMode: .fill)
                    case .failure:
                        placeholder
                    @unknown default:
                        placeholder
                    }
                }
            } else {
                placeholder
            }
        }
        .frame(width: size, height: size)
        .clipShape(RoundedRectangle(cornerRadius: cornerRadius))
    }

    private var placeholder: some View {
        ZStack {
            RoundedRectangle(cornerRadius: cornerRadius)
                .fill(Color(.systemGray5))
            Image(systemName: "music.note")
                .font(.system(size: size * 0.3))
                .foregroundStyle(.secondary)
        }
    }
}

#Preview {
    VStack(spacing: 16) {
        ArtworkImage(url: nil, size: 160)
        ArtworkImage(url: "https://example.com/art.jpg", size: 80)
    }
    .padding()
}
