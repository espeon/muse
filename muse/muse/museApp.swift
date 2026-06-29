//
//  museApp.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import SwiftData
import SwiftUI

@main
struct museApp: App {

    @State private var authManager: AuthManager
    @State private var playerEngine: PlayerEngine
    @State private var remote: RemoteClient

    init() {
        let auth = AuthManager()
        self._authManager = State(initialValue: auth)
        self._playerEngine = State(initialValue: PlayerEngine())
        // The RemoteClient is long-lived (holds a WebSocket), unlike
        // APIClient which is recreated per body evaluation. Its
        // closures capture the auth instance by weak ref so they
        // always read the current serverURL / auth header.
        self._remote = State(initialValue: RemoteClient(
            serverURL: { [weak auth] in auth?.serverURL },
            authHeader: { [weak auth] in auth?.makeAuthHeader() }
        ))
    }

    var sharedModelContainer: ModelContainer = {
        do {
            return try ModelContainer(for: Schema([]))
        } catch {
            fatalError("Could not create ModelContainer: \(error)")
        }
    }()

    var body: some Scene {
        WindowGroup {
            RootViewContainer()
                .environment(authManager)
                .environment(playerEngine)
                .environment(remote)
                .onOpenURL { url in
                    if url.scheme == "muse", url.host == "auth", url.path == "/callback" {
                        try? authManager.handleCallback(url: url)
                    }
                }
        }
        .modelContainer(sharedModelContainer)
    }
}

// Wraps RootView and builds a fresh APIClient whenever authManager.serverURL changes.
private struct RootViewContainer: View {
    @Environment(AuthManager.self) private var authManager
    @Environment(PlayerEngine.self) private var playerEngine
    @Environment(RemoteClient.self) private var remote

    var body: some View {
        let auth = authManager
        let api = APIClient(
            baseURL: auth.serverURL,
            authToken: { auth.makeAuthHeader() },
            refreshToken: { await auth.handleUnauthorized() }
        )
        let umi = UmiClient(baseURL: auth.umiURL)

        RootView()
            .environment(playerEngine)
            .environment(authManager)
            .environment(remote)
            .environment(\.apiClient, api)
            .environment(\.umiClient, umi)
            .onAppear { wireAndStart() }
            .onChange(of: authManager.isAuthenticated) { _, _ in updateConnection() }
            .onChange(of: authManager.serverURL) { _, _ in updateConnection() }
    }

    /// One-time wiring: connect the player engine to the remote
    /// client. Idempotent. Runs on first appearance.
    private func wireAndStart() {
        playerEngine.remote = remote
        remote.onActivePlayerChanged = { [weak playerEngine] isActive in
            playerEngine?.handleActivePlayerChange(isActive)
        }
        remote.onIncomingCommand = { [weak playerEngine] command in
            playerEngine?.handleRemoteCommand(command)
        }
        updateConnection()
    }

    /// Start the connection when authenticated with a server URL;
    /// shut it down otherwise.
    private func updateConnection() {
        let connected = authManager.isAuthenticated && !authManager.serverURL.isEmpty
        if connected {
            remote.start()
        } else {
            remote.shutdown()
        }
    }
}
