package vg.nat.muse.net

import android.content.Context
import android.net.Uri
import androidx.security.crypto.EncryptedSharedPreferences
import androidx.security.crypto.MasterKey
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.request.get
import io.ktor.client.request.header
import io.ktor.client.request.parameter
import io.ktor.client.request.post
import io.ktor.http.HttpHeaders
import io.ktor.serialization.kotlinx.json.json
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.withTimeout
import kotlinx.serialization.Serializable
import java.time.Instant

sealed class AuthException(message: String) : Exception(message) {
    data object NoServerUrl : AuthException("Please enter a server URL first.")
    data object CallbackInvalid : AuthException("Invalid authentication callback.")
    data object NoRefreshToken : AuthException("No refresh token available.")
    data object Unauthorized : AuthException("Session expired. Please log in again.")
    class LoginFailed(message: String) : AuthException("Login failed: $message")
}

class AuthManager(private val appContext: Context) {

    private val masterKey: MasterKey = MasterKey.Builder(appContext)
        .setKeyScheme(MasterKey.KeyScheme.AES256_GCM)
        .build()

    private val securePrefs = EncryptedSharedPreferences.create(
        appContext,
        SECURE_FILE,
        masterKey,
        EncryptedSharedPreferences.PrefKeyEncryptionScheme.AES256_SIV,
        EncryptedSharedPreferences.PrefValueEncryptionScheme.AES256_GCM,
    )

    private val plainPrefs = appContext.getSharedPreferences(PLAIN_FILE, Context.MODE_PRIVATE)

    var serverUrl: String
        get() = plainPrefs.getString(KEY_SERVER_URL, "") ?: ""
        set(value) {
            plainPrefs.edit().putString(KEY_SERVER_URL, value).apply()
        }

    var umiUrl: String
        get() = plainPrefs.getString(KEY_UMI_URL, DEFAULT_UMI_URL) ?: DEFAULT_UMI_URL
        set(value) {
            plainPrefs.edit().putString(KEY_UMI_URL, value).apply()
        }

    private val _isAuthenticated = MutableStateFlow(false)
    val isAuthenticated: StateFlow<Boolean> = _isAuthenticated.asStateFlow()

    private val authHttp = HttpClient {
        install(ContentNegotiation) { json(museJson) }
        expectSuccess = false
    }

    private var loginCallback: CompletableDeferred<Uri>? = null

    init {
        loadStoredTokens()
    }

    fun authHeader(): String? =
        securePrefs.getString(KEY_SESSION_TOKEN, null)?.let { "$AUTH_SCHEME$it" }

    private fun refreshHeader(): String? =
        securePrefs.getString(KEY_REFRESH_TOKEN, null)?.let { "Bearer $it" }

    private fun sessionExpiry(): Instant? =
        securePrefs.getString(KEY_SESSION_EXPIRY, null)?.toLongOrNull()?.let { Instant.ofEpochMilli(it) }

    private fun loadStoredTokens() {
        val token = securePrefs.getString(KEY_SESSION_TOKEN, null)
        val expiry = sessionExpiry()
        if (!token.isNullOrEmpty() && expiry != null && expiry.isAfter(Instant.now())) {
            _isAuthenticated.value = true
        }
    }

    private fun storeTokens(
        sessionToken: String,
        sessionExpiry: Instant,
        refreshToken: String? = null,
        refreshExpiry: Instant? = null,
    ) {
        securePrefs.edit()
            .putString(KEY_SESSION_TOKEN, sessionToken)
            .putString(KEY_SESSION_EXPIRY, sessionExpiry.toEpochMilli().toString())
            .apply()
        if (refreshToken != null) {
            securePrefs.edit().putString(KEY_REFRESH_TOKEN, refreshToken).apply()
        }
        if (refreshExpiry != null) {
            securePrefs.edit().putString(KEY_REFRESH_EXPIRY, refreshExpiry.toEpochMilli().toString())
                .apply()
        }
        _isAuthenticated.value = true
    }

    suspend fun beginLogin(): String {
        if (serverUrl.isEmpty()) throw AuthException.NoServerUrl
        loginCallback = CompletableDeferred()
        val response = authHttp.get("${serverUrl.trimEnd('/')}/api/v1/auth/login") {
            parameter("platform", "mobile")
        }
        if (response.status.value != 200) {
            throw AuthException.LoginFailed("server returned ${response.status.value}")
        }
        val loginResponse: LoginResponse = response.body()
        return loginResponse.url
    }

    suspend fun awaitCallback() {
        val deferred = loginCallback ?: return
        withTimeout(LOGIN_TIMEOUT_MS) { deferred.await() }
    }

    internal fun onRedirect(uri: Uri) {
        try {
            handleCallback(uri)
        } catch (e: AuthException) {
            loginCallback?.completeExceptionally(e)
            loginCallback = null
            return
        }
        loginCallback?.complete(uri)
        loginCallback = null
    }

    fun handleCallback(uri: Uri) {
        val token = uri.getQueryParameter(PARAM_SESSION_TOKEN) ?: throw AuthException.CallbackInvalid
        val expiryStr = uri.getQueryParameter(PARAM_SESSION_EXPIRY) ?: throw AuthException.CallbackInvalid
        val expirySeconds = expiryStr.toDoubleOrNull() ?: throw AuthException.CallbackInvalid
        val sessionExpiry = Instant.ofEpochSecond(
            expirySeconds.toLong(),
            ((expirySeconds - expirySeconds.toLong()) * 1_000_000_000).toLong(),
        )
        val refreshToken = uri.getQueryParameter(PARAM_REFRESH_TOKEN)
        val refreshExpirySeconds = uri.getQueryParameter(PARAM_REFRESH_EXPIRY)?.toDoubleOrNull()
        val refreshExpiry = refreshExpirySeconds?.let {
            Instant.ofEpochSecond(it.toLong(), ((it - it.toLong()) * 1_000_000_000).toLong())
        }
        storeTokens(token, sessionExpiry, refreshToken, refreshExpiry)
    }

    suspend fun refresh() {
        val refreshHeader = refreshHeader() ?: throw AuthException.NoRefreshToken
        if (serverUrl.isEmpty()) throw AuthException.NoServerUrl
        val response = authHttp.post("${serverUrl.trimEnd('/')}/api/v1/auth/refresh") {
            header(HttpHeaders.Authorization, refreshHeader)
        }
        if (response.status.value == 401) {
            logout()
            throw AuthException.Unauthorized
        }
        if (response.status.value != 200) {
            throw AuthException.LoginFailed("refresh failed with ${response.status.value}")
        }
        val result: RefreshResponse = response.body()
        val newExpiry = Instant.ofEpochSecond(result.expiry.toLong())
        storeTokens(result.sessionToken, newExpiry)
    }

    suspend fun handleUnauthorized(): Boolean {
        val expiry = sessionExpiry() ?: return false
        if (!expiry.isAfter(Instant.now().plusSeconds(REFRESH_WINDOW_SECONDS))) {
            return try {
                refresh()
                true
            } catch (_: Exception) {
                logout()
                false
            }
        }
        return false
    }

    fun logout() {
        securePrefs.edit().clear().apply()
        _isAuthenticated.value = false
    }

    @Serializable
    private data class LoginResponse(val url: String)

    @Serializable
    private data class RefreshResponse(val sessionToken: String, val expiry: Int)

    private companion object {
        const val SECURE_FILE = "muse_secure"
        const val PLAIN_FILE = "muse_prefs"
        const val DEFAULT_UMI_URL = "https://umi.nat.vg"
        const val KEY_SERVER_URL = "server_url"
        const val KEY_UMI_URL = "umi_url"
        const val KEY_SESSION_TOKEN = "session_token"
        const val KEY_SESSION_EXPIRY = "session_expiry"
        const val KEY_REFRESH_TOKEN = "refresh_token"
        const val KEY_REFRESH_EXPIRY = "refresh_expiry"
        const val PARAM_SESSION_TOKEN = "session_token"
        const val PARAM_SESSION_EXPIRY = "session_expiry"
        const val PARAM_REFRESH_TOKEN = "refresh_token"
        const val PARAM_REFRESH_EXPIRY = "refresh_expiry"
        const val AUTH_SCHEME = "Bearer authjs.session-token:"
        const val REFRESH_WINDOW_SECONDS = 5L * 60
        const val LOGIN_TIMEOUT_MS = 5L * 60 * 1000
    }
}
