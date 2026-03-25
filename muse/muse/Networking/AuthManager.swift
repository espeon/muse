//
//  AuthManager.swift
//  muse
//
//  Created by Natalie on 3/24/26.
//

import AuthenticationServices
import Foundation
import Observation

// MARK: - KeychainHelper

enum KeychainHelper {
    static func save(_ value: String, forKey key: String) {
        guard let data = value.data(using: .utf8) else { return }
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: key,
            kSecValueData: data,
            kSecAttrAccessible: kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly,
        ]
        SecItemDelete(query as CFDictionary)
        SecItemAdd(query as CFDictionary, nil)
    }

    static func load(forKey key: String) -> String? {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: key,
            kSecReturnData: true,
            kSecMatchLimit: kSecMatchLimitOne,
        ]
        var result: AnyObject?
        let status = SecItemCopyMatching(query as CFDictionary, &result)
        guard status == errSecSuccess,
            let data = result as? Data,
            let string = String(data: data, encoding: .utf8)
        else { return nil }
        return string
    }

    static func delete(forKey key: String) {
        let query: [CFString: Any] = [
            kSecClass: kSecClassGenericPassword,
            kSecAttrAccount: key,
        ]
        SecItemDelete(query as CFDictionary)
    }
}

// MARK: - AuthManager

@Observable
final class AuthManager: NSObject {

    // MARK: - Keys

    private enum Keys {
        static let serverURL = "muse.serverURL"
        static let umiURL = "muse.umiURL"
        static let sessionToken = "muse.sessionToken"
        static let sessionExpiry = "muse.sessionExpiry"
        static let refreshToken = "muse.refreshToken"
        static let refreshExpiry = "muse.refreshExpiry"
    }

    // MARK: - Properties

    var serverURL: String {
        didSet {
            UserDefaults.standard.set(serverURL, forKey: Keys.serverURL)
        }
    }

    var umiURL: String {
        didSet {
            UserDefaults.standard.set(umiURL, forKey: Keys.umiURL)
        }
    }

    var isAuthenticated: Bool = false
    var sessionToken: String?
    var sessionExpiry: Date?
    var refreshToken: String?
    var refreshExpiry: Date?

    private var authSession: ASWebAuthenticationSession?

    // MARK: - Init

    override init() {
        self.serverURL = UserDefaults.standard.string(forKey: Keys.serverURL) ?? ""
        self.umiURL = UserDefaults.standard.string(forKey: Keys.umiURL) ?? "https://umi.muse.moe"
        super.init()
        loadStoredTokens()
    }

    // MARK: - Token Storage

    private func loadStoredTokens() {
        sessionToken = KeychainHelper.load(forKey: Keys.sessionToken)
        refreshToken = KeychainHelper.load(forKey: Keys.refreshToken)

        if let expiryStr = KeychainHelper.load(forKey: Keys.sessionExpiry),
            let expiryVal = Double(expiryStr)
        {
            sessionExpiry = Date(timeIntervalSince1970: expiryVal)
        }

        if let expiryStr = KeychainHelper.load(forKey: Keys.refreshExpiry),
            let expiryVal = Double(expiryStr)
        {
            refreshExpiry = Date(timeIntervalSince1970: expiryVal)
        }

        // Authenticated if we have a session token that hasn't expired
        if let token = sessionToken, !token.isEmpty,
            let expiry = sessionExpiry, expiry > Date()
        {
            isAuthenticated = true
        } else if refreshToken != nil {
            // Try to refresh on next ensureValidToken call
            isAuthenticated = false
        }
    }

    private func storeTokens(
        sessionToken: String, sessionExpiry: Date, refreshToken: String? = nil,
        refreshExpiry: Date? = nil
    ) {
        self.sessionToken = sessionToken
        self.sessionExpiry = sessionExpiry

        KeychainHelper.save(sessionToken, forKey: Keys.sessionToken)
        KeychainHelper.save(String(sessionExpiry.timeIntervalSince1970), forKey: Keys.sessionExpiry)

        if let rt = refreshToken {
            self.refreshToken = rt
            KeychainHelper.save(rt, forKey: Keys.refreshToken)
        }
        if let re = refreshExpiry {
            self.refreshExpiry = re
            KeychainHelper.save(String(re.timeIntervalSince1970), forKey: Keys.refreshExpiry)
        }

        isAuthenticated = true
    }

    // MARK: - Auth Header

    func makeAuthHeader() -> String? {
        guard let token = sessionToken else { return nil }
        return "Bearer authjs.session-token:\(token)"
    }

    func makeRefreshHeader() -> String? {
        guard let token = refreshToken else { return nil }
        return "Bearer \(token)"
    }

    // MARK: - Login

    func login(from anchor: ASPresentationAnchor) async throws {
        guard !serverURL.isEmpty else {
            throw AuthError.noServerURL
        }

        let loginURL = URL(string: "\(serverURL)/api/v1/auth/login?platform=mobile")!
        var request = URLRequest(url: loginURL)
        request.httpMethod = "GET"

        let (data, response) = try await URLSession.shared.data(for: request)

        guard let httpResponse = response as? HTTPURLResponse, httpResponse.statusCode == 200 else {
            throw AuthError.loginFailed("Server returned an error")
        }

        struct LoginResponse: Decodable {
            let url: String
        }

        let loginResponse = try JSONDecoder().decode(LoginResponse.self, from: data)

        guard let oidcURL = URL(string: loginResponse.url) else {
            throw AuthError.loginFailed("Invalid OIDC URL")
        }

        return try await withCheckedThrowingContinuation { continuation in
            let session = ASWebAuthenticationSession(
                url: oidcURL,
                callbackURLScheme: "muse"
            ) { [weak self] callbackURL, error in
                if let error = error {
                    if (error as? ASWebAuthenticationSessionError)?.code == .canceledLogin {
                        continuation.resume(throwing: AuthError.cancelled)
                    } else {
                        continuation.resume(
                            throwing: AuthError.loginFailed(error.localizedDescription))
                    }
                    return
                }

                guard let callbackURL = callbackURL else {
                    continuation.resume(throwing: AuthError.loginFailed("No callback URL"))
                    return
                }

                Task { @MainActor [weak self] in
                    do {
                        try self?.handleCallback(url: callbackURL)
                        continuation.resume()
                    } catch {
                        continuation.resume(throwing: error)
                    }
                }
            }

            session.presentationContextProvider = self
            session.prefersEphemeralWebBrowserSession = false
            self.authSession = session
            session.start()
        }
    }

    // MARK: - Callback Handling

    func handleCallback(url: URL) throws {
        guard let components = URLComponents(url: url, resolvingAgainstBaseURL: false),
            let queryItems = components.queryItems
        else {
            throw AuthError.callbackInvalid
        }

        var params: [String: String] = [:]
        for item in queryItems {
            if let value = item.value {
                params[item.name] = value
            }
        }

        guard let sessionToken = params["session_token"],
            let sessionExpiryStr = params["session_expiry"],
            let sessionExpiryUnix = Double(sessionExpiryStr)
        else {
            throw AuthError.callbackInvalid
        }

        let sessionExpiry = Date(timeIntervalSince1970: sessionExpiryUnix)
        let refreshToken = params["refresh_token"]

        var refreshExpiry: Date? = nil
        if let refreshExpiryStr = params["refresh_expiry"],
            let refreshExpiryUnix = Double(refreshExpiryStr)
        {
            refreshExpiry = Date(timeIntervalSince1970: refreshExpiryUnix)
        }

        storeTokens(
            sessionToken: sessionToken,
            sessionExpiry: sessionExpiry,
            refreshToken: refreshToken,
            refreshExpiry: refreshExpiry
        )
    }

    // MARK: - Token Refresh

    func refresh() async throws {
        guard let refreshHeader = makeRefreshHeader() else {
            throw AuthError.noRefreshToken
        }

        guard !serverURL.isEmpty else {
            throw AuthError.noServerURL
        }

        let url = URL(string: "\(serverURL)/api/v1/auth/refresh")!
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue(refreshHeader, forHTTPHeaderField: "Authorization")

        let (data, response) = try await URLSession.shared.data(for: request)

        guard let httpResponse = response as? HTTPURLResponse else {
            throw AuthError.networkError
        }

        if httpResponse.statusCode == 401 {
            logout()
            throw AuthError.unauthorized
        }

        guard httpResponse.statusCode == 200 else {
            throw AuthError.loginFailed("Refresh failed with status \(httpResponse.statusCode)")
        }

        let decoder = JSONDecoder()
        decoder.keyDecodingStrategy = .convertFromSnakeCase

        struct RefreshResponse: Decodable {
            let sessionToken: String
            let expiry: Int
        }

        let result = try decoder.decode(RefreshResponse.self, from: data)
        let newExpiry = Date(timeIntervalSince1970: Double(result.expiry))

        storeTokens(sessionToken: result.sessionToken, sessionExpiry: newExpiry)
    }

    /// Called when a 401 is received. Attempts to refresh the token and returns true if successful.
    @MainActor
    func handleUnauthorized() async -> Bool {
        guard let expiry = sessionExpiry else {
            return false
        }

        // If token is expired or expiring soon, try to refresh
        if expiry.timeIntervalSinceNow < 5 * 60 {
            do {
                try await refresh()
                return true
            } catch {
                logout()
                return false
            }
        }
        return false
    }

    // MARK: - Ensure Valid Token

    func ensureValidToken() async throws {
        if let expiry = sessionExpiry, expiry.timeIntervalSinceNow > 5 * 60 {
            // Token valid for more than 5 minutes
            return
        }

        // Try to refresh
        try await refresh()
    }

    // MARK: - Logout

    func logout() {
        sessionToken = nil
        sessionExpiry = nil
        refreshToken = nil
        refreshExpiry = nil
        isAuthenticated = false

        KeychainHelper.delete(forKey: Keys.sessionToken)
        KeychainHelper.delete(forKey: Keys.sessionExpiry)
        KeychainHelper.delete(forKey: Keys.refreshToken)
        KeychainHelper.delete(forKey: Keys.refreshExpiry)
    }
}

// MARK: - ASWebAuthenticationPresentationContextProviding

extension AuthManager: ASWebAuthenticationPresentationContextProviding {
    func presentationAnchor(for session: ASWebAuthenticationSession) -> ASPresentationAnchor {
        // Return the first connected window scene's window
        let scenes = UIApplication.shared.connectedScenes
        let windowScene = scenes.first as? UIWindowScene
        let window = windowScene?.windows.first ?? UIWindow()
        return window
    }
}

// MARK: - AuthError

enum AuthError: LocalizedError {
    case noServerURL
    case loginFailed(String)
    case callbackInvalid
    case cancelled
    case noRefreshToken
    case networkError
    case unauthorized

    var errorDescription: String? {
        switch self {
        case .noServerURL:
            return "Please enter a server URL first."
        case .loginFailed(let msg):
            return "Login failed: \(msg)"
        case .callbackInvalid:
            return "Invalid authentication callback."
        case .cancelled:
            return "Login was cancelled."
        case .noRefreshToken:
            return "No refresh token available."
        case .networkError:
            return "A network error occurred."
        case .unauthorized:
            return "Session expired. Please log in again."
        }
    }
}
