//
//  LoginView.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import SwiftUI

struct LoginView: View {
    @Environment(AuthManager.self) private var authManager

    @State private var serverURL: String = ""
    @State private var isLoading: Bool = false
    @State private var errorMessage: String?

    var body: some View {
        VStack(spacing: 40) {
            Spacer()

            // Logo / Name
            VStack(spacing: 12) {
                Image(systemName: "music.note.house.fill")
                    .font(.system(size: 72))
                    .foregroundStyle(.tint)

                Text("Muse")
                    .font(.largeTitle)
                    .fontWeight(.bold)

                Text("Your personal music server")
                    .font(.subheadline)
                    .foregroundStyle(.secondary)
            }

            Spacer()

            // Server URL + Sign In
            VStack(spacing: 16) {
                VStack(alignment: .leading, spacing: 8) {
                    Text("Server URL")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .textCase(.uppercase)

                    TextField("https://music.example.com", text: $serverURL)
                        .textFieldStyle(.roundedBorder)
                        .autocorrectionDisabled()
                        .textInputAutocapitalization(.never)
                        .keyboardType(.URL)
                }

                if let errorMessage = errorMessage {
                    Text(errorMessage)
                        .font(.caption)
                        .foregroundStyle(.red)
                        .multilineTextAlignment(.center)
                }

                Button {
                    signIn()
                } label: {
                    HStack {
                        if isLoading {
                            ProgressView()
                                .progressViewStyle(.circular)
                                .tint(.white)
                        }
                        Text(isLoading ? "Signing in..." : "Sign In")
                            .fontWeight(.semibold)
                    }
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 14)
                    .background(Color.accentColor, in: RoundedRectangle(cornerRadius: 12))
                    .foregroundStyle(.white)
                }
                .disabled(serverURL.trimmingCharacters(in: .whitespaces).isEmpty || isLoading)
                .animation(.easeInOut(duration: 0.2), value: isLoading)
            }
            .padding(.horizontal, 32)

            Spacer()
        }
        .onAppear {
            serverURL = authManager.serverURL
        }
    }

    private func signIn() {
        guard !isLoading else { return }

        let trimmed = serverURL.trimmingCharacters(in: .whitespaces)
        authManager.serverURL = trimmed

        isLoading = true
        errorMessage = nil

        Task { @MainActor in
            do {
                // Get the key window for presentation
                let scenes = UIApplication.shared.connectedScenes
                let windowScene = scenes.first as? UIWindowScene
                let window = windowScene?.windows.first ?? UIWindow()

                try await authManager.login(from: window)
            } catch AuthError.cancelled {
                // User cancelled, don't show error
            } catch {
                errorMessage = error.localizedDescription
            }
            isLoading = false
        }
    }
}

#Preview {
    LoginView()
        .environment(AuthManager())
}
