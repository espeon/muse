package vg.nat.muse.lyrics

import android.util.Log
import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.request.header
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.http.ContentType
import io.ktor.http.contentType
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonArray
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.jsonArray
import kotlinx.serialization.json.jsonObject
import kotlinx.serialization.json.jsonPrimitive

private const val TAG = "LlmTranslator"

@Serializable
private data class ChatMessage(val role: String, val content: String)

@Serializable
private data class ChatRequest(
    val model: String,
    val messages: List<ChatMessage>,
    val temperature: Double = 0.3,
)

@Serializable
private data class ChatChoice(val message: ChatMessage)

@Serializable
private data class ChatResponse(val choices: List<ChatChoice>)

class LlmTranslator(private val json: Json = Json { ignoreUnknownKeys = true }) {

    private val client = HttpClient(io.ktor.client.engine.okhttp.OkHttp) {}

    suspend fun translate(
        texts: List<String>,
        targetLanguage: String,
        endpoint: String,
        apiKey: String,
        model: String,
    ): List<String> {
        val inputArray = texts.map { JsonPrimitive(it) }
        val inputJson = JsonArray(inputArray).toString()

        val systemPrompt =
            """You are a professional lyric translator. You will receive a JSON array of lyric lines, in their original sequential order. Translate each line into $targetLanguage,
taking into account surrounding lines for context. Try to make it flow well in the target language, even if that means deviating from a literal translation. Preserve any rhymes or poetic devices where possible,
but prioritize naturalness and emotional impact in the target language. Meaning should be preserved on each line.

Your response must be ONLY a JSON array of strings, with exactly ${texts.size} elements, in the same order as the input. No markdown, no explanation, just the array.

These are song lyrics presented in the order they are sung. Preserve that order exactly; do not rearrange, merge, or skip lines.

Example input: ["hello world", "goodbye"]
Example output: ["hola mundo", "adiós"]"""

        val requestBody = ChatRequest(
            model = model,
            messages = listOf(
                ChatMessage("system", systemPrompt),
                ChatMessage("user", inputJson),
            ),
        )

        val responseBody: String = try {
            val response = client.post(endpoint) {
                contentType(ContentType.Application.Json)
                header("Authorization", "Bearer $apiKey")
                setBody(json.encodeToString(ChatRequest.serializer(), requestBody))
            }
            response.body()
        } catch (e: Exception) {
            Log.e(TAG, "HTTP request failed", e)
            return texts
        }

        val chatResponse = try {
            json.decodeFromString(ChatResponse.serializer(), responseBody)
        } catch (e: Exception) {
            Log.e(TAG, "Failed to parse chat response: $responseBody", e)
            return texts
        }

        val content = chatResponse.choices.firstOrNull()?.message?.content
        if (content == null) {
            Log.w(TAG, "No content in chat response")
            return texts
        }

        return parseTranslatedArray(content, texts)
    }

    private fun parseTranslatedArray(content: String, fallback: List<String>): List<String> {
        // Strip markdown fences if present
        val cleaned = content.trim()
            .removePrefix("```json").removePrefix("```")
            .removeSuffix("```")
            .trim()

        return try {
            val parsed = json.parseToJsonElement(cleaned)
            val array = extractArray(parsed)
            if (array == null) {
                Log.w(TAG, "Could not extract array from: $cleaned")
                return fallback
            }
            fallback.indices.map { i ->
                array.getOrNull(i)?.jsonPrimitive?.content?.takeIf { it.isNotEmpty() } ?: fallback[i]
            }
        } catch (e: Exception) {
            Log.e(TAG, "Failed to parse translated array: $cleaned", e)
            fallback
        }
    }

    private fun extractArray(element: kotlinx.serialization.json.JsonElement): JsonArray? {
        return when (element) {
            is JsonArray -> element
            is JsonObject -> {
                // Model returned {"lines": [...]} or similar — grab the first array value
                element.values.firstOrNull { it is JsonArray }?.jsonArray
            }

            else -> null
        }
    }
}
