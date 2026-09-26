package ai.suncode.mobile.ui.theme

import ai.suncode.mobile.domain.ThemePreference
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

private val LightColors = lightColorScheme(
    primary = Color(0xFF2C3742),
    onPrimary = Color.White,
    primaryContainer = Color(0xFFE7EDF1),
    onPrimaryContainer = Color(0xFF17202A),
    secondary = Color(0xFF4F6D82),
    background = Color(0xFFF3F5F7),
    surface = Color.White,
    surfaceVariant = Color(0xFFF8FAFC),
    onSurface = Color(0xFF17202A),
    onSurfaceVariant = Color(0xFF44515E),
    outline = Color(0xFFD5DDE4),
    error = Color(0xFFB6463F),
)

private val DarkColors = darkColorScheme(
    primary = Color(0xFFD9E0E6),
    onPrimary = Color(0xFF101317),
    primaryContainer = Color(0xFF20262D),
    onPrimaryContainer = Color(0xFFEDF0F3),
    secondary = Color(0xFF9FB3C3),
    background = Color(0xFF0D0F12),
    surface = Color(0xFF121519),
    surfaceVariant = Color(0xFF181C21),
    onSurface = Color(0xFFEDF0F3),
    onSurfaceVariant = Color(0xFFA7AFB9),
    outline = Color(0xFF292F36),
    error = Color(0xFFE68A83),
)

@Composable
fun SunCodeTheme(preference: ThemePreference, content: @Composable () -> Unit) {
    val dark = when (preference) {
        ThemePreference.SYSTEM -> isSystemInDarkTheme()
        ThemePreference.LIGHT -> false
        ThemePreference.DARK -> true
    }
    MaterialTheme(colorScheme = if (dark) DarkColors else LightColors, content = content)
}
