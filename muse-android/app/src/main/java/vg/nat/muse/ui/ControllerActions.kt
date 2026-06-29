package vg.nat.muse.ui

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.key.Key
import androidx.compose.ui.input.key.KeyEventType
import androidx.compose.ui.input.key.key
import androidx.compose.ui.input.key.onKeyEvent
import androidx.compose.ui.input.key.type

@Composable
fun ControllerActions(
    hasGamepad: Boolean,
    onDismiss: () -> Unit,
    onTogglePlayer: () -> Unit,
    onSwitchTab: (direction: Int) -> Unit,
    modifier: Modifier = Modifier,
    content: @Composable () -> Unit,
) {
    Box(
        modifier = modifier
            .fillMaxSize()
            .onKeyEvent { event ->
                if (!hasGamepad || event.type != KeyEventType.KeyDown) return@onKeyEvent false

                when (event.key) {
                    Key(KeyB) -> { onDismiss(); true }
                    Key(KeyX) -> { onTogglePlayer(); true }
                    Key(KeyL1), Key(KeyR1) -> {
                        val dir = if (event.key == Key(KeyR1)) 1 else -1
                        onSwitchTab(dir)
                        true
                    }
                    else -> false
                }
            },
    ) {
        content()
    }
}

private val KeyB = android.view.KeyEvent.KEYCODE_BUTTON_B
private val KeyX = android.view.KeyEvent.KEYCODE_BUTTON_X
private val KeyL1 = android.view.KeyEvent.KEYCODE_BUTTON_L1
private val KeyR1 = android.view.KeyEvent.KEYCODE_BUTTON_R1
