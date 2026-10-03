package ai.suncode.mobile.remote

actual fun secureRandomBytes(size: Int): ByteArray = ByteArray(size).also { java.security.SecureRandom().nextBytes(it) }
