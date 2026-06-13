package vg.nat.muse.net

import io.ktor.client.HttpClient
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.request.header
import io.ktor.client.request.parameter
import io.ktor.client.request.request
import io.ktor.client.request.setBody
import io.ktor.client.statement.HttpResponse
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpMethod
import io.ktor.serialization.kotlinx.json.json
import kotlinx.serialization.KSerializer
import kotlinx.serialization.json.JsonElement
import kotlinx.serialization.serializer
import kotlinx.serialization.json.add
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.put
import kotlinx.serialization.json.putJsonArray
import java.net.URLEncoder

sealed class ApiException(message: String) : Exception(message) {
    data object Unauthorized : ApiException("Unauthorized. Please log in again.")
    data object NotFound : ApiException("The requested resource was not found.")
    class ServerError(val status: Int, val body: String) :
        ApiException("Server error ($status): $body")
    class Network(cause: Throwable) : ApiException("Network error: ${cause.message}")
}

class ApiClient internal constructor(
    private val authManager: AuthManager,
    private val http: HttpClient,
) {
    constructor(authManager: AuthManager) : this(
        authManager,
        HttpClient {
            install(ContentNegotiation) { json(museJson) }
            expectSuccess = false
        },
    )

    private fun baseUrl(): String = authManager.serverUrl.trimEnd('/')

    private fun authHeader(): String? = authManager.authHeader()

    private suspend fun send(
        fullPath: String,
        method: HttpMethod,
        params: List<Pair<String, String>>,
        body: JsonElement?,
    ): HttpResponse = try {
        http.request(fullPath) {
            this.method = method
            params.forEach { (key, value) -> parameter(key, value) }
            authHeader()?.let { header(HttpHeaders.Authorization, it) }
            if (body != null) {
                header(HttpHeaders.ContentType, ContentType.Application.Json.toString())
                setBody(body)
            }
        }
    } catch (cause: Throwable) {
        throw ApiException.Network(cause)
    }

    private suspend inline fun <reified T> perform(
        path: String,
        method: HttpMethod = HttpMethod.Get,
        params: List<Pair<String, String>> = emptyList(),
        body: JsonElement? = null,
        retrying: Boolean = true,
    ): T = performWithRetry(path, method, params, body, retrying, serializer())

    private suspend fun <T> performWithRetry(
        path: String,
        method: HttpMethod,
        params: List<Pair<String, String>>,
        body: JsonElement?,
        retrying: Boolean,
        serializer: KSerializer<T>,
    ): T {
        val response = send("${baseUrl()}$path", method, params, body)
        return when (val status = response.status.value) {
            in 200..299 -> museJson.decodeFromString(serializer, response.bodyAsText())
            401 -> {
                if (retrying && authManager.handleUnauthorized()) {
                    performWithRetry(path, method, params, body, retrying = false, serializer)
                } else {
                    throw ApiException.Unauthorized
                }
            }
            404 -> throw ApiException.NotFound
            else -> throw ApiException.ServerError(status, response.bodyAsText())
        }
    }

    private suspend fun performEmpty(
        path: String,
        method: HttpMethod = HttpMethod.Get,
        params: List<Pair<String, String>> = emptyList(),
        body: JsonElement? = null,
        retrying: Boolean = true,
    ) {
        val response = send("${baseUrl()}$path", method, params, body)
        when (val status = response.status.value) {
            in 200..299 -> Unit
            401 -> {
                if (retrying && authManager.handleUnauthorized()) {
                    performEmpty(path, method, params, body, retrying = false)
                } else {
                    throw ApiException.Unauthorized
                }
            }
            404 -> throw ApiException.NotFound
            else -> throw ApiException.ServerError(status, response.bodyAsText())
        }
    }

    fun artUrl(id: String, width: Int = 400, height: Int = 400): String =
        "${baseUrl()}/api/v1/art/$id?width=$width&height=$height&format=webp"

    suspend fun fetchHome(): List<HomeRow> = perform("/api/v1/home/")

    suspend fun fetchAlbums(
        cursor: Int = 0,
        limit: Int = 50,
        filter: String? = null,
        sortby: String = "album",
        dir: String = "asc",
    ): AllAlbumsPartial {
        val params = buildList {
            add("cursor" to cursor.toString())
            add("limit" to limit.toString())
            add("sortby" to sortby)
            add("dir" to dir)
            if (!filter.isNullOrEmpty()) add("filter" to filter)
        }
        return perform("/api/v1/album", params = params)
    }

    suspend fun fetchAlbum(id: Int): Album = perform("/api/v1/album/$id")

    suspend fun fetchArtists(
        cursor: Int = 0,
        limit: Int = 50,
        filter: String? = null,
    ): AllArtistsPartial {
        val params = buildList {
            add("cursor" to cursor.toString())
            add("limit" to limit.toString())
            if (!filter.isNullOrEmpty()) add("filter" to filter)
        }
        return perform("/api/v1/artist", params = params)
    }

    suspend fun fetchArtist(id: Int): Artist = perform("/api/v1/artist/$id")

    suspend fun fetchTracks(
        cursor: Int = 0,
        limit: Int = 50,
        lossless: Boolean? = null,
    ): TracksResponse {
        val params = buildList {
            add("cursor" to cursor.toString())
            add("limit" to limit.toString())
            if (lossless != null) add("lossless" to lossless.toString())
        }
        return perform("/api/v1/tracks", params = params)
    }

    suspend fun fetchTrack(id: Int): Track = perform("/api/v1/track/$id")

    suspend fun searchSongs(query: String): List<SearchResult> {
        val encoded = URLEncoder.encode(query, "UTF-8").replace("+", "%20")
        return perform("/api/v1/search/$encoded")
    }

    suspend fun fetchHlsProfiles(): List<HlsProfile> = perform("/api/v1/hls/profiles")

    suspend fun signTrack(id: Int, codec: String? = null, dps: String? = null): SignResult {
        val params = buildList {
            if (codec != null) add("codec" to codec)
            if (dps != null) add("dps" to dps)
        }
        return perform("/api/v1/track/$id/sign", params = params)
    }

    suspend fun batchSignTracks(
        ids: List<Int>,
        codec: String? = null,
        dps: String? = null,
        mode: String? = null,
    ): List<SignResult> {
        val body = buildJsonObject {
            putJsonArray("ids") { ids.forEach { add(it.toString()) } }
            codec?.let { put("codec", it) }
            dps?.let { put("dps", it) }
            mode?.let { put("mode", it) }
        }
        return perform("/api/v1/tracks/sign", HttpMethod.Post, body = body)
    }

    suspend fun toggleLike(trackId: Int): LikedResponse =
        perform("/api/v1/track/$trackId/like", HttpMethod.Post)

    suspend fun scrobble(trackId: Int) {
        performEmpty("/api/v1/track/$trackId/scrobble")
    }

    suspend fun setPlaying(trackId: Int) {
        performEmpty("/api/v1/track/$trackId/play")
    }

    suspend fun fetchHistory(limit: Int = 50, offset: Int = 0): List<PlayHistoryEntry> {
        val params = listOf(
            "limit" to limit.toString(),
            "offset" to offset.toString(),
        )
        return perform("/api/v1/history", params = params)
    }

    suspend fun fetchPlaylists(): List<PlaylistSummary> = perform("/api/v1/playlist")

    suspend fun createPlaylist(name: String, description: String? = null): PlaylistSummary {
        val body = buildJsonObject {
            put("name", name)
            description?.let { put("description", it) }
        }
        return perform("/api/v1/playlist", HttpMethod.Post, body = body)
    }

    suspend fun fetchPlaylist(id: Int): PlaylistDetail = perform("/api/v1/playlist/$id")

    suspend fun updatePlaylist(
        id: Int,
        name: String? = null,
        description: String? = null,
    ): PlaylistSummary {
        val body = buildJsonObject {
            name?.let { put("name", it) }
            description?.let { put("description", it) }
        }
        return perform("/api/v1/playlist/$id", HttpMethod.Put, body = body)
    }

    suspend fun deletePlaylist(id: Int) {
        performEmpty("/api/v1/playlist/$id", HttpMethod.Delete)
    }

    suspend fun addTrackToPlaylist(playlistId: Int, songId: Int): PlaylistTrack {
        val body = buildJsonObject { put("song_id", songId) }
        return perform("/api/v1/playlist/$playlistId/tracks", HttpMethod.Post, body = body)
    }

    suspend fun removeTrackFromPlaylist(playlistId: Int, itemId: Int) {
        performEmpty("/api/v1/playlist/$playlistId/tracks/$itemId", HttpMethod.Delete)
    }

    suspend fun reorderPlaylistTrack(playlistId: Int, itemId: Int, afterItemId: Int?) {
        val body = buildJsonObject {
            if (afterItemId != null) put("after_item_id", afterItemId)
        }
        performEmpty(
            "/api/v1/playlist/$playlistId/tracks/$itemId/position",
            HttpMethod.Put,
            body = body,
        )
    }

    suspend fun fetchGenres(): List<GenreEntry> = perform("/api/v1/genres")
}
