@file:OptIn(kotlinx.cinterop.ExperimentalForeignApi::class)
package ai.suncode.mobile.remote

import kotlinx.cinterop.addressOf
import kotlinx.cinterop.usePinned
import platform.Security.SecRandomCopyBytes
import platform.Security.kSecRandomDefault

actual fun secureRandomBytes(size: Int): ByteArray = ByteArray(size).also { bytes ->
    bytes.usePinned { require(SecRandomCopyBytes(kSecRandomDefault, size.toULong(), it.addressOf(0)) == 0) { "Unable to generate secure random bytes" } }
}
