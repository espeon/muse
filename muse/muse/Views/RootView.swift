import SwiftUI

struct RootView: View {
    @Environment(AuthManager.self) private var authManager
    @Environment(PlayerEngine.self) private var playerEngine
    @Environment(RemoteClient.self) private var remote

    @State private var selectedTab: AppTab = .home
    @Namespace private var playerNamespace

    private enum AppTab: Hashable {
        case home, library, search, settings
    }

    var body: some View {
        Group {
            if authManager.isAuthenticated {
                makeTabView()
            } else {
                LoginView()
            }
        }
    }

    private func makeTabView() -> some View {
        @Bindable var player = playerEngine

        return TabView(selection: $selectedTab) {
            Tab("Home", systemImage: "house", value: AppTab.home) {
                NavigationStack { HomeView() }
            }
            Tab("Library", systemImage: "music.note.list", value: AppTab.library) {
                NavigationStack { LibraryView() }
            }
            Tab("Search", systemImage: "magnifyingglass", value: AppTab.search, role: .search) {
                NavigationStack { SearchView() }
            }
            Tab("Settings", systemImage: "gear", value: AppTab.settings) {
                NavigationStack { SettingsView() }
            }
        }
        .tabBarMinimizeBehavior(.onScrollDown)
        .tabViewBottomAccessory {
            PlayerControlBar(
                player: playerEngine,
                remote: remote,
                isExpanded: $player.showFullPlayer
            )
            .matchedTransitionSource(id: "nowPlaying", in: playerNamespace)
            .onTapGesture {
                player.showFullPlayer.toggle()
            }
        }
        .fullScreenCover(isPresented: $player.showFullPlayer) {
            ScrollView {

            }.safeAreaInset(edge: .top, spacing: 0) {
                VStack {
                    NowPlayingSheetView(
                        player: playerEngine,
                        remote: remote,
                        dismiss: { player.showFullPlayer = false }
                    )
                }
                .navigationTransition(.zoom(sourceID: "nowPlaying", in: playerNamespace))

            }
            .frame(maxWidth: .infinity, maxHeight: .infinity)
            .background(.background)
        }
    }
}
