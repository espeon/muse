package vg.nat.muse.ui.player

import androidx.activity.BackEventCompat
import androidx.activity.compose.PredictiveBackHandler
import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.Spring
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.foundation.gestures.detectVerticalDragGestures
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.layout.layout
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.launch
import kotlin.math.roundToInt

@Composable
fun PlayerSheet(
    visible: Boolean,
    onDismiss: () -> Unit,
    content: @Composable () -> Unit,
) {
    if (!visible) return
    BoxWithConstraints(Modifier.fillMaxSize()) {
        val heightPx = constraints.maxHeight.toFloat()
        val offsetAnim = remember(heightPx) { Animatable(heightPx) }
        val scope = rememberCoroutineScope()

        LaunchedEffect(heightPx) {
            offsetAnim.animateTo(0f, tween(280))
        }

        PredictiveBackHandler(enabled = true) { progress: Flow<BackEventCompat> ->
            try {
                progress.collect { ev ->
                    offsetAnim.snapTo((heightPx * ev.progress).coerceIn(0f, heightPx))
                }
                offsetAnim.animateTo(heightPx, spring(dampingRatio = Spring.DampingRatioMediumBouncy))
                onDismiss()
            } catch (e: CancellationException) {
                offsetAnim.animateTo(0f, spring())
                throw e
            }
        }

        Box(
            Modifier
                .fillMaxSize()
                .pointerInput(heightPx) {
                    detectVerticalDragGestures(
                        onVerticalDrag = { _, dragAmount ->
                            val next = (offsetAnim.value + dragAmount).coerceIn(0f, heightPx)
                            scope.launch { offsetAnim.snapTo(next) }
                        },
                        onDragEnd = {
                            scope.launch {
                                val target = if (offsetAnim.value > heightPx / 2f) heightPx else 0f
                                offsetAnim.animateTo(
                                    target,
                                    spring(dampingRatio = Spring.DampingRatioMediumBouncy),
                                )
                                if (target >= heightPx) onDismiss()
                            }
                        },
                        onDragCancel = {
                            scope.launch {
                                offsetAnim.animateTo(
                                    if (offsetAnim.value > heightPx / 2f) heightPx else 0f,
                                    spring(),
                                )
                            }
                        },
                    )
                }
                .verticalOffset(offsetAnim.value),
        ) {
            content()
        }
    }
}

private fun Modifier.verticalOffset(y: Float): Modifier = layout { measurable, constraints ->
    val placeable = measurable.measure(constraints)
    layout(placeable.width, placeable.height) {
        placeable.placeRelative(0, y.roundToInt())
    }
}
