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
    @Environment(RemoteClient.self) private var remote

    @State private var serverURL: String
    @State private var showingLogoutAlert = false
    @State private var llmAPIKey: String
    @State private var llmBaseURL: String
    @State private var llmModel: String

    @State private var lastfmConnected = false
    @State private var lastfmUsername: String?
    @State private var pendingToken: String?
    @State private var pairingStep: PairingStep = .idle
    @State private var lastfmError: String?

    enum PairingStep {
        case idle
        case awaitingApproval
        case connecting
    }

    init() {
        _serverURL = State(
            initialValue: UserDefaults.standard.string(forKey: "muse.serverURL") ?? "")
        _llmAPIKey = State(
            initialValue: UserDefaults.standard.string(forKey: "muse.llm.apiKey") ?? "")
        _llmBaseURL = State(
            initialValue: UserDefaults.standard.string(forKey: "muse.llm.baseURL") ?? "https://openrouter.ai/api/v1")
        _llmModel = State(
            initialValue: UserDefaults.standard.string(forKey: "muse.llm.model") ?? "openai/gpt-4o-mini")
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

                Section("Last.fm") {
                    switch pairingStep {
                    case .idle:
                        if lastfmConnected {
                            HStack {
                                Label("Connected", systemImage: "checkmark.circle.fill")
                                    .foregroundStyle(.green)
                                if let user = lastfmUsername {
                                    Spacer()
                                    Text(user)
                                        .foregroundStyle(.secondary)
                                        .font(.caption)
                                }
                            }
                            Button(role: .destructive) {
                                Task { await disconnectLastfm() }
                            } label: {
                                Text("Disconnect")
                            }
                        } else {
                            Button {
                                Task { await beginLastfmPairing() }
                            } label: {
                                Label("Connect to Last.fm", systemImage: "music.note")
                            }
                        }
                    case .awaitingApproval:
                        VStack(alignment: .leading, spacing: 8) {
                            Text(
                                "Complete the authorization in your browser, then return here and tap Continue."
                            )
                            .font(.caption)
                            .foregroundStyle(.secondary)
                            HStack {
                                Button("Continue") {
                                    Task { await finishLastfmPairing() }
                                }
                                .buttonStyle(.borderedProminent)
                                Button("Cancel", role: .cancel) {
                                    pendingToken = nil
                                    pairingStep = .idle
                                }
                            }
                        }
                    case .connecting:
                        HStack {
                            ProgressView()
                            Text("Connecting…")
                        }
                    }

                    if let err = lastfmError {
                        Text(err)
                            .font(.caption)
                            .foregroundStyle(.red)
                    }
                }

                Section("Devices") {
                    HStack {
                        Text("Connection")
                        Spacer()
                        switch remote.connectionState {
                        case .disconnected:
                            Text("Disconnected").foregroundStyle(.secondary)
                        case .connecting:
                            Text("Connecting…").foregroundStyle(.secondary)
                        case .connected:
                            Label("Connected", systemImage: "checkmark.circle.fill")
                                .foregroundStyle(.green)
                        case .failed(let m):
                            Text(m).foregroundStyle(.red).lineLimit(1)
                        }
                    }

                    if remote.activeDeviceId == nil, authManager.isAuthenticated {
                        Button {
                            Task { await remote.sendTransfer(to: remote.myDeviceId) }
                        } label: {
                            Label("Play on this device", systemImage: "play.circle")
                        }
                    }

                    NavigationLink {
                        DevicesView()
                    } label: {
                        HStack {
                            Label("Manage Devices", systemImage: "speaker.wave.2")
                            Spacer()
                            Text("\(remote.devices.count)")
                                .foregroundStyle(.secondary)
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

                Section("LLM Translation") {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("API Key")
                            .font(.headline)
                        SecureField("sk-or-v1-...", text: $llmAPIKey)
                            .textInputAutocapitalization(.never)
                            .autocorrectionDisabled()
                            .onChange(of: llmAPIKey) { _, newValue in
                                UserDefaults.standard.set(newValue, forKey: "muse.llm.apiKey")
                            }
                    }

                    VStack(alignment: .leading, spacing: 8) {
                        Text("Base URL")
                            .font(.headline)
                        TextField("https://openrouter.ai/api/v1", text: $llmBaseURL)
                            .textInputAutocapitalization(.never)
                            .autocorrectionDisabled()
                            .onChange(of: llmBaseURL) { _, newValue in
                                UserDefaults.standard.set(newValue, forKey: "muse.llm.baseURL")
                            }
                        Text("OpenRouter, OpenAI, or any OpenAI-compatible provider")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }

                    VStack(alignment: .leading, spacing: 8) {
                        Text("Model")
                            .font(.headline)
                        TextField("openai/gpt-4o-mini", text: $llmModel)
                            .textInputAutocapitalization(.never)
                            .autocorrectionDisabled()
                            .onChange(of: llmModel) { _, newValue in
                                UserDefaults.standard.set(newValue, forKey: "muse.llm.model")
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
                if let me = try? await apiClient.fetchMe() {
                    lastfmConnected = me.lastfmConnected
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
                    remote.shutdown()
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

    // MARK: - Last.fm Pairing

    private func beginLastfmPairing() async {
        lastfmError = nil
        do {
            let resp = try await apiClient.fetchLastfmToken()
            pendingToken = resp.token
            pairingStep = .awaitingApproval
            guard let url = URL(string: resp.url) else {
                lastfmError = "Invalid authorization URL"
                pairingStep = .idle
                return
            }
            await UIApplication.shared.open(url)
        } catch {
            lastfmError = error.localizedDescription
            pairingStep = .idle
        }
    }

    private func finishLastfmPairing() async {
        guard let token = pendingToken else {
            pairingStep = .idle
            return
        }
        pairingStep = .connecting
        do {
            let resp = try await apiClient.completeLastfmSession(token: token)
            lastfmConnected = true
            lastfmUsername = resp.username
            pendingToken = nil
            pairingStep = .idle
        } catch {
            lastfmError = error.localizedDescription
            pairingStep = .awaitingApproval
        }
    }

    private func disconnectLastfm() async {
        lastfmError = nil
        do {
            try await apiClient.disconnectLastfm()
            lastfmConnected = false
            lastfmUsername = nil
        } catch {
            lastfmError = error.localizedDescription
        }
    }
}

#Preview {
    SettingsView()
        .environment(AuthManager())
        .environment(PlayerEngine())
}
