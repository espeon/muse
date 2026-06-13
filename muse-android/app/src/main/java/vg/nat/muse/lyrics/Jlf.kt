package vg.nat.muse.lyrics

import kotlinx.serialization.KSerializer
import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonDecoder
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonPrimitive

object SecondsToMsSerializer : KSerializer<Int> {
    override val descriptor: SerialDescriptor =
        PrimitiveSerialDescriptor("SecondsToMs", PrimitiveKind.INT)

    override fun deserialize(decoder: Decoder): Int {
        val json = decoder as? JsonDecoder
            ?: return decoder.decodeInt()
        return when (val el = json.decodeJsonElement()) {
            is JsonNull -> 0
            is JsonPrimitive -> {
                val d = if (el.isString) el.content.toDoubleOrNull() ?: 0.0 else el.content.toDoubleOrNull() ?: 0.0
                (d * 1000).toInt()
            }
            else -> 0
        }
    }

    override fun serialize(encoder: Encoder, value: Int) {
        encoder.encodeInt(value / 1000)
    }
}

@Serializable
data class Jlf(
    val lines: SyncedLines,
    val richsync: SyncedRich? = null,
    val metadata: SyncedMetadata? = null,
    val source: String,
    val name: String? = null,
    val message: String? = null,
)

@Serializable
data class SyncedLines(
    val lines: List<SyncedLine>,
    @Serializable(with = SecondsToMsSerializer::class) val linesEnd: Int = 0,
)

@Serializable
data class SyncedLine(
    @Serializable(with = SecondsToMsSerializer::class) val time: Int,
    val text: String = "",
    val translation: String? = null,
)

@Serializable
data class SyncedMetadata(
    @SerialName("MxmId") val mxmId: String? = null,
    @SerialName("ITunesId") val iTunesId: String? = null,
    @SerialName("SpotifyId") val spotifyId: String? = null,
    @SerialName("Artist") val artist: String,
    @SerialName("Title") val title: String,
    @SerialName("Album") val album: String,
    @SerialName("Copyright") val copyright: String? = null,
)

@Serializable
data class SyncedRich(
    @Serializable(with = SecondsToMsSerializer::class) val totalTime: Int,
    val sections: List<SyncedRichSection>,
    val agents: List<SyncedRichAgent> = emptyList(),
)

@Serializable
data class SyncedRichAgent(val type: String, val id: String)

@Serializable
data class SyncedRichSection(
    @Serializable(with = SecondsToMsSerializer::class) val timeStart: Int,
    @Serializable(with = SecondsToMsSerializer::class) val timeEnd: Int,
    val lines: List<SyncedRichLine>,
)

@Serializable
data class SyncedRichLine(
    @Serializable(with = SecondsToMsSerializer::class) val timeStart: Int,
    @Serializable(with = SecondsToMsSerializer::class) val timeEnd: Int,
    val text: String,
    val segments: List<SyncedRichLineSegment>,
    val agent: String,
    val bgVox: SyncedRichBackgroundLine? = null,
)

@Serializable
data class SyncedRichBackgroundLine(
    @Serializable(with = SecondsToMsSerializer::class) val timeStart: Int,
    @Serializable(with = SecondsToMsSerializer::class) val timeEnd: Int,
    val text: String,
    val segments: List<SyncedRichLineSegment>,
)

@Serializable
data class SyncedRichLineSegment(
    val text: String,
    @Serializable(with = SecondsToMsSerializer::class) val timeStart: Int,
    @Serializable(with = SecondsToMsSerializer::class) val timeEnd: Int,
)

val umiJson: Json = Json {
    ignoreUnknownKeys = true
    explicitNulls = false
    isLenient = true
}
