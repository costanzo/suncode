package ai.suncode.mobile.remote

import android.content.Context
import android.util.Base64
import ai.suncode.mobile.remote.RemoteHostCredentials
import ai.suncode.mobile.remote.RemoteHostCredentialStore
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

    override suspend fun load(hostId: String): RemoteHostCredentials? = read()[hostId]

    override suspend fun save(hostId: String, credentials: RemoteHostCredentials) {
        val values = read().toMutableMap()
        values[hostId] = credentials
        write(values)
    }

    override suspend fun clear(hostId: String) {
        val values = read().toMutableMap()
        values.remove(hostId)
        write(values)
    }

    private fun write(values: Map<String, RemoteHostCredentials>) {
        val iv = ByteArray(12).also(SecureRandom()::nextBytes)
        val cipher = Cipher.getInstance(TRANSFORMATION)
        cipher.init(Cipher.ENCRYPT_MODE, key(), GCMParameterSpec(128, iv))
        val encoded = json.encodeToString(RemoteHostCredentialStore.serializer(), RemoteHostCredentialStore(values))
        val encrypted = cipher.doFinal(encoded.encodeToByteArray())
        preferences.edit().putString(STORAGE_KEY, Base64.encodeToString(iv + encrypted, Base64.NO_WRAP)).apply()
    }

    private fun read(): Map<String, RemoteHostCredentials> {
        val encoded = preferences.getString(STORAGE_KEY, null) ?: return emptyMap()
        return runCatching {
            val combined = Base64.decode(encoded, Base64.NO_WRAP)
            require(combined.size > 12)
            val cipher = Cipher.getInstance(TRANSFORMATION)
            cipher.init(Cipher.DECRYPT_MODE, key(), GCMParameterSpec(128, combined.copyOfRange(0, 12)))
            json.decodeFromString<RemoteHostCredentialStore>(String(cipher.doFinal(combined.copyOfRange(12, combined.size)))).hosts
        }.getOrDefault(emptyMap())
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
