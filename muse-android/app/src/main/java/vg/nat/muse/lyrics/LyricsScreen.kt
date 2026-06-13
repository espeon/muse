package vg.nat.muse.lyrics

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.AnimationVector1D
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
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
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.withFrameNanos
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

private data class SegRender(val startMs: Int, val endMs: Int, val text: String)
private data class RenderLine(val startMs: Int, val endMs: Int, val text: String, val segments: List<SegRender>?)

private val LyricStyle = TextStyle(fontWeight = FontWeight.Bold, fontSize = 28.sp, lineHeight = 34.sp)

@Composable
fun LyricsScreen(
    jlf: Jlf,
    positionMsProvider: () -> Long,
    isPlaying: Boolean,
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

    val smoothMs = remember { Animatable(positionMsProvider().toFloat()) }
    LaunchedEffect(isPlaying) {
        while (isPlaying) {
            withFrameNanos { it }
            smoothMs.snapTo(positionMsProvider().toFloat())
        }
        smoothMs.snapTo(positionMsProvider().toFloat())
    }

    val activeIndex by remember(lines) {
        derivedStateOf { activeIndexFor(lines, smoothMs.value.toInt()) }
    }
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
                    smoothMs = smoothMs,
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
    smoothMs: Animatable<Float, AnimationVector1D>,
    onSeek: (Int) -> Unit,
) {
    val dimAlpha = (0.4f / (1f + 0.3f * max(0, distanceFromActive))).coerceAtLeast(0.12f)

    Box(
        modifier = Modifier
            .fillMaxWidth()
            .clickable { onSeek(line.startMs) }
            .padding(vertical = 14.dp),
        contentAlignment = Alignment.CenterStart,
    ) {
        when {
            line.text.isEmpty() -> InstrumentalDots(isActive = isActive)
            isActive && line.segments != null -> SyllabicLine(line.segments, smoothMs)
            else -> Text(
                text = line.text,
                color = if (isActive) Color.White else Color.White.copy(alpha = dimAlpha),
                style = LyricStyle,
                modifier = Modifier.fillMaxWidth(),
            )
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
private fun SyllabicLine(segments: List<SegRender>, smoothMs: Animatable<Float, AnimationVector1D>) {
    val activeSeg by remember(segments) {
        derivedStateOf { activeSegIndex(segments, smoothMs.value.toInt()) }
    }
    FlowRow(
        horizontalArrangement = Arrangement.spacedBy(0.dp),
        verticalArrangement = Arrangement.spacedBy(0.dp),
    ) {
        segments.forEachIndexed { i, seg ->
            when {
                i < activeSeg -> Text(seg.text, color = Color.White, style = LyricStyle)
                i == activeSeg -> Box(contentAlignment = Alignment.CenterStart) {
                    Text(seg.text, color = Color.White.copy(alpha = 0.28f), style = LyricStyle)
                    Text(
                        text = seg.text,
                        color = Color.White,
                        style = LyricStyle,
                        modifier = Modifier.drawWithContent {
                            val progress = segProgress(seg, smoothMs.value.toInt())
                            clipRect(0f, 0f, size.width * progress, size.height, ClipOp.Intersect) {
                                this@drawWithContent.drawContent()
                            }
                        },
                    )
                }
                else -> Text(seg.text, color = Color.White.copy(alpha = 0.3f), style = LyricStyle)
            }
        }
    }
}

@Composable
private fun InstrumentalDots(isActive: Boolean) {
    val sizeDp = if (isActive) 14.dp else 9.dp
    Row(horizontalArrangement = Arrangement.spacedBy(sizeDp)) {
        repeat(3) {
            Box(
                Modifier
                    .size(sizeDp)
                    .background(Color.White.copy(alpha = if (isActive) 0.9f else 0.3f), CircleShape),
            )
        }
    }
}

private fun segProgress(seg: SegRender, ms: Int): Float {
    val duration = max(1, seg.endMs - seg.startMs)
    return ((ms - seg.startMs).toFloat() / duration).coerceIn(0f, 1f)
}

private fun activeSegIndex(segments: List<SegRender>, ms: Int): Int {
    var lo = 0
    var hi = segments.size
    while (lo < hi) {
        val mid = (lo + hi) / 2
        if (segments[mid].startMs <= ms) lo = mid + 1 else hi = mid
    }
    return (lo - 1).coerceIn(-1, segments.size - 1)
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
            val segs = line.segments.takeIf { it.isNotEmpty() }?.map {
                SegRender(it.timeStart, it.timeEnd, it.text)
            }
            RenderLine(line.timeStart, end, line.text, segs)
        }
    }
    val plain = jlf.lines.lines
    return plain.mapIndexed { i, line ->
        val end = plain.getOrNull(i + 1)?.time ?: (line.time + 6000)
        RenderLine(line.time, end, line.text, null)
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
