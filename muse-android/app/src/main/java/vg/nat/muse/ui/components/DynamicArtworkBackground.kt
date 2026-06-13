package vg.nat.muse.ui.components

import android.graphics.BitmapFactory
import android.graphics.Paint
import android.graphics.RuntimeShader
import android.os.Build
import androidx.compose.foundation.Canvas
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.drawIntoCanvas
import androidx.compose.ui.graphics.nativeCanvas
import androidx.palette.graphics.Palette
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.isActive
import kotlinx.coroutines.withContext
import java.net.HttpURLConnection
import java.net.URL

private data class PaletteColors(val c1: Color, val c2: Color, val c3: Color)

@Composable
fun DynamicArtworkBackground(
    artworkUrl: String?,
    modifier: Modifier = Modifier,
) {
    val fallback = remember { PaletteColors(Color(0xFF2A2D33), Color(0xFF3B4252), Color(0xFF111316)) }
    var colors by remember(artworkUrl) { mutableStateOf(fallback) }

    LaunchedEffect(artworkUrl) {
        if (artworkUrl == null) {
            colors = fallback
            return@LaunchedEffect
        }
        val extracted = withContext(Dispatchers.IO) { extractPalette(artworkUrl) }
        if (extracted != null) colors = extracted
    }

    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
        ShaderBackground(colors, modifier)
    } else {
        GradientFallback(colors, modifier)
    }
}

@Composable
private fun ShaderBackground(colors: PaletteColors, modifier: Modifier) {
    val shader = remember { RuntimeShader(BlobsAgsl) }
    val paint = remember { Paint() }
    var time by remember { mutableFloatStateOf(0f) }

    LaunchedEffect(Unit) {
        var last = 0L
        while (isActive) {
            val now = withFrameNanos { it }
            if (last != 0L) time += (now - last) / 1e9f
            last = now
        }
    }

    Canvas(modifier) {
        shader.setFloatUniform("uResolution", size.width, size.height)
        shader.setFloatUniform("uTime", time)
        shader.setFloatUniform("uColor1", colors.c1.red, colors.c1.green, colors.c1.blue)
        shader.setFloatUniform("uColor2", colors.c2.red, colors.c2.green, colors.c2.blue)
        shader.setFloatUniform("uColor3", colors.c3.red, colors.c3.green, colors.c3.blue)
        paint.shader = shader
        paint.isAntiAlias = true
        drawIntoCanvas { it.nativeCanvas.drawRect(0f, 0f, size.width, size.height, paint) }
    }
}

@Composable
private fun GradientFallback(colors: PaletteColors, modifier: Modifier) {
    Canvas(modifier) {
        drawRect(
            brush = Brush.radialGradient(
                colors = listOf(colors.c1, colors.c3),
                center = Offset(size.width * 0.5f, size.height * 0.38f),
                radius = maxOf(size.width, size.height),
            ),
        )
    }
}

private fun extractPalette(url: String): PaletteColors? = runCatching {
    val conn = URL(url).openConnection() as HttpURLConnection
    conn.connectTimeout = 8000
    conn.readTimeout = 8000
    val bitmap = conn.inputStream.use { BitmapFactory.decodeStream(it) } ?: return null
    conn.disconnect()
    val palette = Palette.from(bitmap).generate()
    PaletteColors(
        c1 = Color(palette.getDominantColor(0xFF3B4252.toInt())),
        c2 = Color(palette.getVibrantColor(palette.getMutedColor(0xFF4C566A.toInt()))),
        c3 = Color(palette.getDarkMutedColor(palette.getDarkVibrantColor(0xFF111316.toInt()))),
    )
}.getOrNull()

private val BlobsAgsl = """
uniform float2 uResolution;
uniform float  uTime;
uniform half3  uColor1;
uniform half3  uColor2;
uniform half3  uColor3;

half4 main(float2 fragCoord) {
    float2 uv = fragCoord / uResolution;

    float2 c1 = float2(0.30 + 0.12 * sin(uTime * 0.30),
                       0.40 + 0.10 * cos(uTime * 0.21));
    float2 c2 = float2(0.70 + 0.11 * cos(uTime * 0.25 + 1.7),
                       0.62 + 0.12 * sin(uTime * 0.27 + 0.5));
    float2 c3 = float2(0.50 + 0.13 * sin(uTime * 0.18 + 2.3),
                       0.50 + 0.08 * cos(uTime * 0.33));

    float d1 = 1.0 / (distance(uv, c1) * 2.4 + 0.45);
    float d2 = 1.0 / (distance(uv, c2) * 2.4 + 0.45);
    float d3 = 1.0 / (distance(uv, c3) * 2.8 + 0.45);

    float total = d1 + d2 + d3;
    half3 col = (uColor1 * d1 + uColor2 * d2 + uColor3 * d3) / total;
    col *= mix(0.72, 1.06, uv.y);

    return half4(col, 1.0);
}
""".trimIndent()
