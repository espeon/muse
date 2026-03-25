//
//  RootView.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import SwiftUI

struct RootView: View {
    @Environment(AuthManager.self) private var authManager
    @Environment(PlayerEngine.self) private var playerEngine

    var body: some View {
        Group {
            if authManager.isAuthenticated {
                mainContent
            } else {
                LoginView()
            }
        }
    }

    private var mainContent: some View {
        @Bindable var player = playerEngine

        return TabView {
            NavigationStack {
                HomeView()
            }
            .tabItem {
                Label("Home", systemImage: "house.fill")
            }

            NavigationStack {
                LibraryView()
            }
            .tabItem {
                Label("Library", systemImage: "music.note.list")
            }

            NavigationStack {
                SearchView()
            }
            .tabItem {
                Label("Search", systemImage: "magnifyingglass")
            }
        }
        .overlay(alignment: .bottom) {
            if playerEngine.currentTrack != nil {
                PlayerControlBar(player: playerEngine, isExpanded: $player.showFullPlayer)
                    .padding(.bottom, 49)  // Approximate tab bar height
            }
        }
        .sheet(isPresented: $player.showFullPlayer) {
            NowPlayingSheetView(
                player: playerEngine,
                dismiss: {
                    player.showFullPlayer = false
                })
        }
    }
}
