//
//  SettingsView.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import SwiftUI

struct SettingsView: View {
    @Environment(\.dismiss) private var dismiss
    @Environment(AuthManager.self) private var authManager
    @Environment(PlayerEngine.self) private var player
    @Environment(\.apiClient) private var apiClient

    @State private var serverURL: String
    @State private var showingLogoutAlert = false

    init() {
        _serverURL = State(
            initialValue: UserDefaults.standard.string(forKey: "muse.serverURL") ?? "")
    }

    var body: some View {
        NavigationView {
            Form {
                Section {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Server URL")
                            .font(.headline)
                        TextField("https://api.muse.lutea.co", text: $serverURL)
                            .textInputAutocapitalization(.never)
                            .keyboardType(.URL)
                            .onChange(of: serverURL) { _, newValue in
                                UserDefaults.standard.set(newValue, forKey: "muse.serverURL")
                                authManager.serverURL = newValue
                            }
                        Text("The URL of your Maki server")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }

                Section {
                    HStack {
                        Text("Authenticated")
                        Spacer()
                        if authManager.isAuthenticated {
                            Label("Yes", systemImage: "checkmark.circle.fill")
                                .foregroundStyle(.green)
                        } else {
                            Label("No", systemImage: "xmark.circle.fill")
                                .foregroundStyle(.red)
                        }
                    }

                    if authManager.isAuthenticated {
                        Button(role: .destructive) {
                            showingLogoutAlert = true
                        } label: {
                            HStack {
                                Image(systemName: "rectangle.portrait.and.arrow.right")
                                Text("Log Out")
                            }
                        }
                    }
                }

                Section("Playback") {
                    Toggle(
                        isOn: Binding(
                            get: { player.useHLS },
                            set: { player.useHLS = $0 }
                        )
                    ) {
                        VStack(alignment: .leading, spacing: 2) {
                            Text("Adaptive Streaming")
                            Text(
                                "Auto-adjusts quality to your internet connection. Takes effect on next track."
                            )
                            .font(.caption)
                            .foregroundStyle(.secondary)
                        }
                    }

                    if player.useHLS {
                        Picker(
                            "Quality",
                            selection: Binding(
                                get: { player.selectedProfile ?? "auto" },
                                set: { player.setQuality($0 == "auto" ? nil : $0) }
                            )
                        ) {
                            Text("Auto").tag("auto")
                            ForEach(player.hlsProfiles) { profile in
                                Text(profile.displayName).tag(profile.name)
                            }
                        }
                    }
                }

                Section {
                    VStack(alignment: .leading, spacing: 4) {
                        Text("App Version")
                        Text(
                            Bundle.main.infoDictionary?["CFBundleShortVersionString"] as? String
                                ?? "Unknown"
                        )
                        .foregroundStyle(.secondary)
                    }

                    VStack(alignment: .leading, spacing: 4) {
                        Text("Build")
                        Text(Bundle.main.infoDictionary?["CFBundleVersion"] as? String ?? "Unknown")
                            .foregroundStyle(.secondary)
                    }
                }
            }
            .task {
                if player.hlsProfiles.isEmpty {
                    player.hlsProfiles = (try? await apiClient.fetchHLSProfiles()) ?? []
                }
            }
            .navigationTitle("Settings")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Done") {
                        dismiss()
                    }
                }
            }
            .alert("Log Out", isPresented: $showingLogoutAlert) {
                Button("Cancel", role: .cancel) {}
                Button("Log Out", role: .destructive) {
                    authManager.logout()
                    dismiss()
                }
            } message: {
                Text(
                    "Are you sure you want to log out? You will need to sign in again to access your music."
                )
            }
        }
    }
}

#Preview {
    SettingsView()
        .environment(AuthManager())
        .environment(PlayerEngine())
}
