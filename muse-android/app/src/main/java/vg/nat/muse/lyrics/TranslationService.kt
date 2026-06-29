package vg.nat.muse.lyrics

import android.content.Context
import com.google.mlkit.nl.languageid.LanguageIdentification
import com.google.mlkit.nl.translate.TranslateLanguage
import com.google.mlkit.nl.translate.Translation
import com.google.mlkit.nl.translate.Translator
import com.google.mlkit.nl.translate.TranslatorOptions
import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.suspendCancellableCoroutine
import java.util.Locale
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException

enum class TranslationBackend { MLKIT, LLM }

data class TranslationResult(
    val targetLanguage: String,
    val lines: List<String>,
)

class TranslationService(private val context: Context) {

    private val prefs = context.getSharedPreferences("translation_prefs", 0)
    private val llmTranslator = LlmTranslator()
    private val translators = mutableMapOf<Pair<String, String>, Translator>()

    var backend: TranslationBackend
        get() = prefs.getString("backend", null)
            ?.let { runCatching { TranslationBackend.valueOf(it) }.getOrNull() }
            ?: TranslationBackend.MLKIT
        set(value) = prefs.edit().putString("backend", value.name).apply()

    val isConfigured: Boolean
        get() = when (backend) {
            TranslationBackend.MLKIT -> true
            TranslationBackend.LLM -> apiKey.isNotBlank() && endpoint.isNotBlank()
        }

    var endpoint: String
        get() = prefs.getString("llm_endpoint", "") ?: ""
        set(value) = prefs.edit().putString("llm_endpoint", value).apply()

    var apiKey: String
        get() = prefs.getString("llm_api_key", "") ?: ""
        set(value) = prefs.edit().putString("llm_api_key", value).apply()

    var model: String
        get() = prefs.getString("llm_model", "openai/gpt-4o-mini") ?: "openai/gpt-4o-mini"
        set(value) = prefs.edit().putString("llm_model", value).apply()

    fun deviceLanguageCode(): String {
        val locale = Locale.getDefault()
        val tag = locale.toLanguageTag()
        return TranslateLanguage.fromLanguageTag(tag)
            ?: TranslateLanguage.fromLanguageTag(tag.split("-").first())
            ?: TranslateLanguage.ENGLISH
    }

    fun deviceLanguageName(): String {
        val locale = Locale.getDefault()
        return locale.displayLanguage.replaceFirstChar { it.titlecase(locale) }
    }

    suspend fun translate(
        jlf: Jlf,
        targetLanguage: String,
    ): TranslationResult {
        val texts: List<String> = if (jlf.richsync != null) {
            jlf.richsync.sections.flatMap { section -> section.lines.map { it.text } }
        } else {
            jlf.lines.lines.map { it.text }
        }
        if (texts.all { it.isBlank() }) return TranslationResult(targetLanguage, texts)

        return when (backend) {
            TranslationBackend.LLM -> translateWithLlm(texts, targetLanguage)
            TranslationBackend.MLKIT -> translateWithMlKit(texts, targetLanguage)
        }
    }

    suspend fun translateRichBgVox(
        jlf: Jlf,
        targetLanguage: String,
    ): TranslationResult? {
        val rich = jlf.richsync ?: return null
        val bgVoxLines: List<String> = rich.sections.flatMap { section ->
            section.lines.map { it.bgVox?.text ?: "" }
        }
        if (bgVoxLines.all { it.isBlank() }) return null

        return when (backend) {
            TranslationBackend.LLM -> translateWithLlm(bgVoxLines, targetLanguage)
            TranslationBackend.MLKIT -> translateWithMlKit(bgVoxLines, targetLanguage)
        }
    }

    fun cleanup() {
        translators.values.forEach { it.close() }
        translators.clear()
    }

    // --- LLM ---

    private suspend fun translateWithLlm(
        texts: List<String>,
        targetLanguage: String,
    ): TranslationResult {
        if (!isConfigured) return TranslationResult(targetLanguage, texts)
        val translated = llmTranslator.translate(
            texts = texts,
            targetLanguage = targetLanguage,
            endpoint = endpoint,
            apiKey = apiKey,
            model = model,
        )
        return TranslationResult(targetLanguage, translated)
    }

    // --- ML Kit ---

    private suspend fun translateWithMlKit(
        texts: List<String>,
        targetLanguage: String,
    ): TranslationResult {
        val source = detectSource(texts)
            ?: return TranslationResult(targetLanguage, texts)
        return TranslationResult(targetLanguage, translateLines(texts, source, targetLanguage))
    }

    private suspend fun translateLines(
        texts: List<String>,
        source: String,
        target: String,
    ): List<String> = coroutineScope {
        if (source == target) return@coroutineScope texts
        val translator = getOrCreateTranslator(source, target)
        translator.ensureModel()
        texts.map { text ->
            async {
                if (text.isBlank()) text
                else runCatching { translator.translateLine(text) }.getOrElse { text }
            }
        }.awaitAll()
    }

    private suspend fun detectSource(lines: List<String>): String? {
        val sample = lines.filter { it.isNotBlank() }.take(8).joinToString("\n")
        if (sample.isBlank()) return null
        val tag = detectLanguage(sample) ?: return null
        return TranslateLanguage.fromLanguageTag(tag)
    }

    private suspend fun detectLanguage(sample: String): String? =
        suspendCancellableCoroutine { cont ->
            LanguageIdentification.getClient()
                .identifyLanguage(sample)
                .addOnSuccessListener { code -> cont.resume(code.takeUnless { it == "und" }) }
                .addOnFailureListener { cont.resumeWithException(it) }
        }

    private suspend fun Translator.translateLine(text: String): String =
        suspendCancellableCoroutine { cont ->
            translate(text)
                .addOnSuccessListener { cont.resume(it) }
                .addOnFailureListener { cont.resumeWithException(it) }
        }

    private suspend fun Translator.ensureModel() =
        suspendCancellableCoroutine<Unit> { cont ->
            downloadModelIfNeeded()
                .addOnSuccessListener { cont.resume(Unit) }
                .addOnFailureListener { cont.resumeWithException(it) }
        }

    private fun getOrCreateTranslator(source: String, target: String): Translator =
        translators.getOrPut(source to target) {
            val options = TranslatorOptions.Builder()
                .setSourceLanguage(source)
                .setTargetLanguage(target)
                .build()
            Translation.getClient(options)
        }
}
