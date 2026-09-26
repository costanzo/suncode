package ai.suncode.mobile.remote

import android.content.Context
import android.util.Base64
import ai.suncode.mobile.remote.protocol.TokenData
import kotlinx.serialization.json.Json
import java.security.KeyStore
import java.security.SecureRandom
import javax.crypto.Cipher
import javax.crypto.KeyGenerator
import javax.crypto.SecretKey
import javax.crypto.spec.GCMParameterSpec
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyProperties

/** Stores Remote Control credentials encrypted by an Android Keystore AES key. */
class AndroidKeystoreRemoteTokenStore(context: Context) : RemoteTokenStore {
    private val preferences = context.applicationContext.getSharedPreferences(PREFERENCES, Context.MODE_PRIVATE)
    private val json = Json { ignoreUnknownKeys = true }

    override suspend fun accessToken(): String? = read()?.accessToken

    override suspend fun refreshToken(): String? = read()?.refreshToken

    override suspend fun save(tokens: TokenData) {
        val iv = ByteArray(12).also(SecureRandom()::nextBytes)
        val cipher = Cipher.getInstance(TRANSFORMATION)
        cipher.init(Cipher.ENCRYPT_MODE, key(), GCMParameterSpec(128, iv))
        val encrypted = cipher.doFinal(json.encodeToString(TokenData.serializer(), tokens).encodeToByteArray())
        preferences.edit().putString(STORAGE_KEY, Base64.encodeToString(iv + encrypted, Base64.NO_WRAP)).apply()
    }

    override suspend fun clear() {
        preferences.edit().remove(STORAGE_KEY).apply()
    }

    private fun read(): TokenData? {
        val encoded = preferences.getString(STORAGE_KEY, null) ?: return null
        return runCatching {
            val combined = Base64.decode(encoded, Base64.NO_WRAP)
            require(combined.size > 12)
            val cipher = Cipher.getInstance(TRANSFORMATION)
            cipher.init(Cipher.DECRYPT_MODE, key(), GCMParameterSpec(128, combined.copyOfRange(0, 12)))
            json.decodeFromString<TokenData>(String(cipher.doFinal(combined.copyOfRange(12, combined.size))))
        }.getOrNull()
    }

    private fun key(): SecretKey {
        val store = KeyStore.getInstance(ANDROID_KEYSTORE).apply { load(null) }
        (store.getKey(KEY_ALIAS, null) as? SecretKey)?.let { return it }
        val generator = KeyGenerator.getInstance(KeyProperties.KEY_ALGORITHM_AES, ANDROID_KEYSTORE)
        generator.init(
            KeyGenParameterSpec.Builder(
                KEY_ALIAS,
                KeyProperties.PURPOSE_ENCRYPT or KeyProperties.PURPOSE_DECRYPT,
            )
                .setBlockModes(KeyProperties.BLOCK_MODE_GCM)
                .setEncryptionPaddings(KeyProperties.ENCRYPTION_PADDING_NONE)
                .setKeySize(256)
                .build(),
        )
        return generator.generateKey()
    }

    private companion object {
        const val ANDROID_KEYSTORE = "AndroidKeyStore"
        const val KEY_ALIAS = "suncode.mobile.remote.token.v1"
        const val PREFERENCES = "suncode.remote.credentials"
        const val STORAGE_KEY = "encrypted_tokens"
        const val TRANSFORMATION = "AES/GCM/NoPadding"
    }
}
