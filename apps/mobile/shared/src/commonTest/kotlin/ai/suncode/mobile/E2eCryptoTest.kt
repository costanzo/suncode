package ai.suncode.mobile

import ai.suncode.mobile.remote.E2eCrypto
import kotlin.test.Test
import kotlin.test.assertContentEquals

class E2eCryptoTest {
    @Test
    fun roundTripsAes256GcmPayload() {
        val key = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA" // 32 zero bytes, base64url
        val plaintext = "{\"message\":\"hello\"}".encodeToByteArray()
        val encrypted = E2eCrypto.encrypt(key, plaintext)
        assertContentEquals(plaintext, E2eCrypto.decrypt(key, encrypted))
    }
}
