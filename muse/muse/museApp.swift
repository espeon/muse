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

    @State private var authManager = AuthManager()
    @State private var playerEngine = PlayerEngine()

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
            .environment(\.apiClient, api)
            .environment(\.umiClient, umi)
    }
}
