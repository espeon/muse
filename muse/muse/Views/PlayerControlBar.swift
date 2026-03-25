import SwiftUI

struct PlayerControlBar: View {
    let player: PlayerEngine
    @Binding var isExpanded: Bool
    var isTabBarMinimized: Bool = false

    var body: some View {
        HStack {
            if let track = player.currentTrack {
                // Artwork
                if let artUrl = track.artUrl, let url = URL(string: artUrl) {
                    AsyncImage(url: url) { image in
                        image.resizable()
                    } placeholder: {
                        Color.gray
                    }
                    .frame(width: 44, height: 44)
                    .cornerRadius(6)
                } else {
                    Rectangle()
                        .fill(Color.gray)
                        .frame(width: 44, height: 44)
                        .cornerRadius(6)
                }

                // Info
                VStack(alignment: .leading) {
                    Text(track.name)
                        .font(.headline)
                        .lineLimit(1)
                    Text(track.displayArtist)
                        .font(.subheadline)
                        .foregroundColor(.secondary)
                        .lineLimit(1)
                }

                Spacer()

                // Controls
                Button(action: {
                    player.togglePlayPause()
                }) {
                    Image(systemName: player.isPlaying ? "pause.fill" : "play.fill")
                        .font(.title2)
                }
                .padding(.horizontal)
            } else {
                Text("Not Playing")
                    .foregroundColor(.secondary)
                Spacer()
            }
        }
        .padding()
        .background(Material.regular)
        .onTapGesture {
            isExpanded = true
        }
    }
}
