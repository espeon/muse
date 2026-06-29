package vg.nat.muse.ui

import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.border
import androidx.compose.foundation.focusable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.interaction.collectIsFocusedAsState
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.scale
import androidx.compose.ui.focus.FocusRequester
import androidx.compose.ui.focus.focusRequester
import androidx.compose.ui.focus.onFocusChanged
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onPreviewKeyEvent
import androidx.compose.ui.input.key.type
import androidx.compose.ui.unit.dp

private val FocusShape = RoundedCornerShape(12.dp)

@Composable
fun FocusableItem(
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    focusRequester: FocusRequester = remember { FocusRequester() },
    content: @Composable () -> Unit,
) {
    val interactionSource = remember { MutableInteractionSource() }
    val isFocused by interactionSource.collectIsFocusedAsState()

    val scale by animateFloatAsState(
        targetValue = if (isFocused) 1.03f else 1f,
        animationSpec = tween(150),
        label = "focusScale",
    )

    val focusColor = MaterialTheme.colorScheme.primary

    Box(
        modifier = modifier
            .scale(scale)
            .onFocusChanged { }
            .focusRequester(focusRequester)
            .then(
                if (isFocused) Modifier.border(2.dp, focusColor, FocusShape)
                else Modifier.border(2.dp, Color.Transparent, FocusShape),
            )
            .clip(FocusShape)
            .onPreviewKeyEvent { event ->
                if (event.type == KeyEventType.KeyDown &&
                    (event.key == Key(ButtonA) || event.key == Key(ButtonDpadCenter))
                ) {
                    onClick()
                    true
                } else {
                    false
                }
            }
            .focusable(interactionSource = interactionSource),
    ) {
        content()
    }
}

private val ButtonA = android.view.KeyEvent.KEYCODE_BUTTON_A
private val ButtonDpadCenter = android.view.KeyEvent.KEYCODE_DPAD_CENTER
