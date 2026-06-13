package vg.nat.muse.lyrics

import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithContent
import androidx.compose.ui.graphics.ClipOp
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.drawscope.clipRect
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import kotlin.math.max

private data class RenderLine(val startMs: Int, val endMs: Int, val text: String)

@Composable
fun LyricsScreen(
    jlf: Jlf,
    currentTimeMs: Int,
    onSeek: (Int) -> Unit,
    modifier: Modifier = Modifier,
) {
    val lines = remember(jlf) { buildRenderLines(jlf) }
    if (lines.isEmpty()) {
        Box(modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
            Text(
                "No lyrics available",
                color = Color.White.copy(alpha = 0.6f),
                style = MaterialTheme.typography.titleMedium,
            )
        }
        return
    }
    val activeIndex = remember(lines, currentTimeMs) { activeIndexFor(lines, currentTimeMs) }
    val listState = rememberLazyListState()

    BoxWithConstraints(modifier.fillMaxSize()) {
        val density = LocalDensity.current
        val viewportPx = with(density) { maxHeight.toPx() }
        val verticalPadDp = with(density) { (viewportPx * 0.35f).toDp() }

        LaunchedEffect(activeIndex) {
            if (activeIndex >= 0) {
                runCatching {
                    listState.animateScrollToItem(
                        activeIndex,
                        scrollOffset = -(viewportPx * 0.35f).toInt(),
                    )
                }
            }
        }

        LazyColumn(
            state = listState,
            modifier = Modifier.fillMaxSize(),
            contentPadding = PaddingValues(top = verticalPadDp, bottom = verticalPadDp),
        ) {
            itemsIndexed(lines, key = { i, l -> l.startMs * 1000 + i }) { i, line ->
                LyricRow(
                    line = line,
                    isActive = i == activeIndex,
                    distanceFromActive = i - activeIndex,
                    currentTimeMs = currentTimeMs,
                    onSeek = onSeek,
                )
            }
        }
    }
}

@Composable
private fun LyricRow(
    line: RenderLine,
    isActive: Boolean,
    distanceFromActive: Int,
    currentTimeMs: Int,
    onSeek: (Int) -> Unit,
) {
    val duration = max(1, line.endMs - line.startMs)
    val progress = ((currentTimeMs - line.startMs).toFloat() / duration).coerceIn(0f, 1f)
    val sweep by animateFloatAsState(
        targetValue = progress,
        animationSpec = tween(durationMillis = 200, easing = LinearEasing),
        label = "sweep",
    )
    val dimAlpha = (0.4f / (1f + 0.3f * max(0, distanceFromActive))).coerceAtLeast(0.12f)

    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clickable { onSeek(line.startMs) }
            .padding(vertical = 14.dp),
        contentAlignment = Alignment.CenterStart,
    ) {
        if (line.text.isEmpty()) {
            InstrumentalDots(isActive = isActive)
        } else {
            val style = TextStyle(fontWeight = FontWeight.Bold, fontSize = 28.sp, lineHeight = 34.sp)
            Text(
                text = line.text,
                color = Color.White.copy(alpha = if (isActive) 0.28f else dimAlpha),
                style = style,
                modifier = Modifier.fillMaxWidth(),
            )
            if (isActive) {
                Text(
                    text = line.text,
                    color = Color.White,
                    style = style,
                    modifier = Modifier
                        .fillMaxWidth()
                        .drawWithContent {
                            clipRect(0f, 0f, size.width * sweep, size.height, ClipOp.Intersect) {
                                this@drawWithContent.drawContent()
                            }
                        },
                )
            }
        }
    }
}

@Composable
private fun InstrumentalDots(isActive: Boolean) {
    val size by animateFloatAsState(if (isActive) 14f else 9f, label = "dot")
    Row(horizontalArrangement = Arrangement.spacedBy(size.dp)) {
        repeat(3) {
            Box(
                Modifier
                    .size(size.dp)
                    .background(Color.White.copy(alpha = if (isActive) 0.9f else 0.3f), CircleShape),
            )
        }
    }
}

private fun buildRenderLines(jlf: Jlf): List<RenderLine> {
    val rich = jlf.richsync
    if (rich != null) {
        val flat = rich.sections.flatMap { it.lines }
        return flat.mapIndexed { i, line ->
            val end = if (line.timeEnd > 0) {
                line.timeEnd
            } else {
                flat.getOrNull(i + 1)?.timeStart ?: (line.timeStart + 4000)
            }
            RenderLine(line.timeStart, end, line.text)
        }
    }
    val plain = jlf.lines.lines
    return plain.mapIndexed { i, line ->
        val end = plain.getOrNull(i + 1)?.time ?: (line.time + 6000)
        RenderLine(line.time, end, line.text)
    }
}

private fun activeIndexFor(lines: List<RenderLine>, timeMs: Int): Int {
    if (lines.isEmpty()) return -1
    var lo = 0
    var hi = lines.size
    while (lo < hi) {
        val mid = (lo + hi) / 2
        if (lines[mid].startMs <= timeMs) lo = mid + 1 else hi = mid
    }
    return (lo - 1).coerceAtLeast(0)
}
