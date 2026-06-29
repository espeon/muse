import SwiftUI

struct PlayerControlBar: View {
    let player: PlayerEngine
    let remote: RemoteClient
    @Binding var isExpanded: Bool
    var isTabBarMinimized: Bool = false

    var body: some View {
        // Decide what to render. The "other device is playing" branch
        // is read-only with a single "Play here" affordance; the
        // normal branch is the local mini player.
        if let activeId = remote.activeDeviceId,
            activeId != remote.myDeviceId,
            let active = remote.devices.first(where: { $0.deviceId == activeId })
        {
            remotePlayingBar(active: active, isExpanded: $isExpanded)
        } else {
            localBar(isExpanded: $isExpanded)
        }
    }

    @ViewBuilder
    private func localBar(isExpanded: Binding<Bool>) -> some View {
        HStack {
            if let track = player.currentTrack {
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

                VStack(alignment: .leading) {
                    Text(track.name)
                        .font(.subheadline)
                        .lineLimit(1)
                    Text(track.displayArtist)
                        .font(.caption)
                        .foregroundColor(.secondary)
                        .lineLimit(1)
                }

                Spacer()

                Button(action: {
                    player.togglePlayPause()
                }) {
                    Image(systemName: player.isPlaying ? "pause.fill" : "play.fill")
                        .font(.title2)
                        .foregroundStyle(Color.primary)
                }
                .padding(.horizontal)
            } else {
                Text("Not Playing")
                    .foregroundColor(.secondary)
                Spacer()
            }
        }
        .padding()
        .onTapGesture {
            isExpanded.wrappedValue = true
        }
    }

    @ViewBuilder
    private func remotePlayingBar(
        active: RemoteDevice,
        isExpanded: Binding<Bool>
    ) -> some View {
        HStack(spacing: 12) {
            // Cast-style icon
            Image(systemName: "speaker.wave.3.fill")
                .font(.title2)
                .foregroundStyle(.green)
                .frame(width: 44, height: 44)

            VStack(alignment: .leading, spacing: 2) {
                Text("Playing on \(active.name)")
                    .font(.subheadline)
                    .lineLimit(1)
                if let state = remote.lastState, state.isPlaying {
                    Text("Now playing")
                        .font(.caption)
                        .foregroundColor(.secondary)
                } else if remote.lastState != nil {
                    Text("Paused")
                        .font(.caption)
                        .foregroundColor(.secondary)
                } else {
                    Text("Connected")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }

            Spacer()

            Button {
                Task { await remote.sendTransfer(to: remote.myDeviceId) }
            } label: {
                Image(systemName: "iphone.gen3")
                    .font(.title2)
                    .foregroundStyle(Color.primary)
            }
            .padding(.horizontal)
        }
        .padding()
        .onTapGesture {
            isExpanded.wrappedValue = true
        }
    }
}

#Preview("Local Playing") {
    @Previewable @State var expanded = false
    PlayerControlBar(
        player: .preview,
        remote: .previewLocalActive,
        isExpanded: $expanded
    )
}

#Preview("Local Idle") {
    @Previewable @State var expanded = false
    PlayerControlBar(
        player: .previewIdle,
        remote: .previewLocalActive,
        isExpanded: $expanded
    )
}

#Preview("Remote Playing") {
    @Previewable @State var expanded = false
    PlayerControlBar(
        player: .previewIdle,
        remote: .previewRemote,
        isExpanded: $expanded
    )
}
