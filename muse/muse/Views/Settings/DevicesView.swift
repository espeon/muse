//
//  DevicesView.swift
//  muse
//
//  Lists all devices connected to the user's remote session, with
//  actions to transfer playback. Pushed from SettingsView.
//

import SwiftUI

struct DevicesView: View {
    @Environment(RemoteClient.self) private var remote

    var body: some View {
        Form {
            Section {
                if remote.activeDeviceId == nil {
                    HStack {
                        Image(systemName: "speaker.slash")
                            .foregroundStyle(.secondary)
                        Text("Nothing is playing")
                            .foregroundStyle(.secondary)
                    }
                    Button {
                        Task { await remote.sendTransfer(to: remote.myDeviceId) }
                    } label: {
                        Label("Play on this device", systemImage: "play.circle.fill")
                    }
                } else if let active = remote.devices.first(where: { $0.isActivePlayer }) {
                    HStack {
                        Image(systemName: "speaker.wave.3.fill")
                            .foregroundStyle(.green)
                        VStack(alignment: .leading) {
                            Text(active.name).font(.headline)
                            Text("Active player")
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                    }
                    if active.deviceId != remote.myDeviceId {
                        Button {
                            Task { await remote.sendTransfer(to: remote.myDeviceId) }
                        } label: {
                            Label("Transfer to this device", systemImage: "iphone.gen3")
                        }
                    }
                }
            } header: {
                Text("Now Playing On")
            } footer: {
                Text(
                    "Active player is the device that's rendering audio. Other devices can control it and curate its queue."
                )
            }

            Section("This Device") {
                LabeledContent("Name", value: remote.myDeviceName)
                LabeledContent("ID") {
                    Text(remote.myDeviceId)
                        .font(.system(.caption, design: .monospaced))
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                        .truncationMode(.middle)
                }
            }

            Section("All Devices") {
                if remote.devices.isEmpty {
                    Text("No devices connected.")
                        .foregroundStyle(.secondary)
                }
                ForEach(remote.devices) { device in
                    deviceRow(device)
                }
            }
        }
        .navigationTitle("Devices")
        .navigationBarTitleDisplayMode(.inline)
    }

    @ViewBuilder
    private func deviceRow(_ device: RemoteDevice) -> some View {
        let isSelf = device.deviceId == remote.myDeviceId
        HStack {
            Image(systemName: iconName(for: device.kind))
                .foregroundStyle(device.isActivePlayer ? .green : .secondary)
                .frame(width: 28)
            VStack(alignment: .leading) {
                HStack {
                    Text(device.name)
                    if isSelf {
                        Text("(this device)")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
                Text(kindLabel(device.kind))
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            Spacer()
            if device.isActivePlayer {
                Image(systemName: "speaker.wave.3.fill")
                    .foregroundStyle(.green)
            } else if !isSelf {
                Button {
                    Task { await remote.sendTransfer(to: device.deviceId) }
                } label: {
                    Text("Transfer")
                }
                .buttonStyle(.borderless)
            }
        }
    }

    private func iconName(for kind: RemoteDeviceKind) -> String {
        switch kind {
        case .ios: return "iphone.gen3"
        case .android: return "candybarphone"
        case .web: return "globe"
        case .server: return "server.rack"
        }
    }

    private func kindLabel(_ kind: RemoteDeviceKind) -> String {
        switch kind {
        case .ios: return "iPhone / iPad"
        case .android: return "Android"
        case .web: return "Web"
        case .server: return "Server player"
        }
    }
}
