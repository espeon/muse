package vg.nat.muse.net

import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.KSerializer
import kotlinx.serialization.SerializationException
import kotlinx.serialization.descriptors.PrimitiveKind
import kotlinx.serialization.descriptors.PrimitiveSerialDescriptor
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonDecoder
import kotlinx.serialization.json.JsonNamingStrategy
import kotlinx.serialization.json.JsonNull
import kotlinx.serialization.json.JsonPrimitive
import java.time.Instant
import java.time.format.DateTimeParseException

object FlexInstantSerializer : KSerializer<Instant> {
    override val descriptor: SerialDescriptor =
        PrimitiveSerialDescriptor("FlexInstant", PrimitiveKind.STRING)

    override fun deserialize(decoder: Decoder): Instant {
        val jsonDecoder = decoder as? JsonDecoder
            ?: throw SerializationException("FlexInstantSerializer requires a JsonDecoder")
        return when (val element = jsonDecoder.decodeJsonElement()) {
            is JsonNull -> throw SerializationException("Expected a date but got null")
            is JsonPrimitive -> parsePrimitive(element)
            else -> throw SerializationException("Unexpected date element: $element")
        }
    }

    override fun serialize(encoder: Encoder, value: Instant) {
        encoder.encodeString(value.toString())
    }

    private fun parsePrimitive(primitive: JsonPrimitive): Instant {
        if (!primitive.isString) {
            val value = primitive.content.toDoubleOrNull()
                ?: throw SerializationException("Cannot parse date: ${primitive.content}")
            return epochSeconds(value)
        }
        val text = primitive.content
        return try {
            Instant.parse(text)
        } catch (_: DateTimeParseException) {
            text.toDoubleOrNull()?.let { epochSeconds(it) }
                ?: throw SerializationException("Cannot parse date string: $text")
        }
    }

    private fun epochSeconds(value: Double): Instant {
        val whole = value.toLong()
        val nanos = ((value - whole) * 1_000_000_000).toLong()
        return Instant.ofEpochSecond(whole, nanos)
    }
}

@OptIn(ExperimentalSerializationApi::class)
val museJson: Json = Json {
    ignoreUnknownKeys = true
    encodeDefaults = false
    explicitNulls = false
    namingStrategy = JsonNamingStrategy.SnakeCase
}
