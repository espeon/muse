package vg.nat.muse.controller

import android.hardware.input.InputManager
import android.view.InputDevice
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.State
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext

fun isGamepad(device: InputDevice): Boolean =
    device.sources and (InputDevice.SOURCE_GAMEPAD or InputDevice.SOURCE_JOYSTICK) != 0

fun hasGamepad(): Boolean =
    InputDevice.getDeviceIds().any { id -> InputDevice.getDevice(id)?.let(::isGamepad) == true }

@Composable
fun rememberGamepadConnected(): State<Boolean> {
    val context = LocalContext.current
    val state = remember { mutableStateOf(hasGamepad()) }

    DisposableEffect(context) {
        val inputManager = context.getSystemService(InputManager::class.java)
        val listener = object : InputManager.InputDeviceListener {
            override fun onInputDeviceAdded(deviceId: Int) {
                InputDevice.getDevice(deviceId)?.let { device ->
                    if (isGamepad(device)) state.value = true
                }
            }

            override fun onInputDeviceRemoved(deviceId: Int) {
                state.value = hasGamepad()
            }

            override fun onInputDeviceChanged(deviceId: Int) {}
        }
        inputManager.registerInputDeviceListener(listener, null)
        onDispose { inputManager.unregisterInputDeviceListener(listener) }
    }

    return state
}
