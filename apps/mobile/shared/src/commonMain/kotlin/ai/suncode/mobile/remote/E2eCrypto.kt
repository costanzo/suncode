package ai.suncode.mobile.remote

import kotlin.experimental.xor

/** AES-256-GCM wire format used by remote-control: e2e-v1:<base64url(nonce || ciphertext || tag)>. */
internal expect fun secureRandomBytes(size: Int): ByteArray

internal object E2eCrypto {
    private const val PREFIX = "e2e-v1:"
    private const val NONCE_SIZE = 12
    private const val TAG_SIZE = 16

    fun encrypt(keyText: String, plaintext: ByteArray): String {
        val key = decodeBase64Url(keyText)
        require(key.size == 32) { "Remote E2E key must be 32 bytes" }
        val nonce = secureRandomBytes(NONCE_SIZE)
        val aes = Aes256(key)
        val ciphertext = cryptCtr(aes, nonce, plaintext)
        val tag = ghashTag(aes, nonce, ciphertext)
        return PREFIX + encodeBase64Url(nonce + ciphertext + tag)
    }

    fun decrypt(keyText: String, payload: String): ByteArray {
        require(payload.startsWith(PREFIX)) { "Unsupported encrypted payload version" }
        val key = decodeBase64Url(keyText)
        require(key.size == 32) { "Remote E2E key must be 32 bytes" }
        val bytes = decodeBase64Url(payload.removePrefix(PREFIX))
        require(bytes.size >= NONCE_SIZE + TAG_SIZE) { "Encrypted payload is truncated" }
        val nonce = bytes.copyOfRange(0, NONCE_SIZE)
        val tagStart = bytes.size - TAG_SIZE
        val ciphertext = bytes.copyOfRange(NONCE_SIZE, tagStart)
        val tag = bytes.copyOfRange(tagStart, bytes.size)
        val aes = Aes256(key)
        val expected = ghashTag(aes, nonce, ciphertext)
        var mismatch = 0
        for (i in tag.indices) mismatch = mismatch or (tag[i].toInt() xor expected[i].toInt())
        require(mismatch == 0) { "Encrypted payload authentication failed" }
        return cryptCtr(aes, nonce, ciphertext)
    }

    private fun cryptCtr(aes: Aes256, nonce: ByteArray, input: ByteArray): ByteArray {
        val out = ByteArray(input.size)
        var counter = 2
        var offset = 0
        while (offset < input.size) {
            val block = nonce + byteArrayOf((counter ushr 24).toByte(), (counter ushr 16).toByte(), (counter ushr 8).toByte(), counter.toByte())
            val stream = aes.encrypt(block)
            val count = minOf(16, input.size - offset)
            for (i in 0 until count) out[offset + i] = input[offset + i] xor stream[i]
            offset += count
            counter++
        }
        return out
    }

    private fun ghashTag(aes: Aes256, nonce: ByteArray, ciphertext: ByteArray): ByteArray {
        val h = aes.encrypt(ByteArray(16))
        var y = ByteArray(16)
        fun absorb(block: ByteArray) { y = gfMultiply(xor(y, block), h) }
        var offset = 0
        while (offset < ciphertext.size) {
            val block = ByteArray(16)
            ciphertext.copyInto(block, 0, offset, minOf(offset + 16, ciphertext.size))
            absorb(block)
            offset += 16
        }
        val lengths = ByteArray(16)
        val bits = ciphertext.size.toLong() * 8
        for (i in 0 until 8) lengths[15 - i] = (bits ushr (i * 8)).toByte()
        absorb(lengths)
        val j0 = nonce + byteArrayOf(0, 0, 0, 1)
        return xor(aes.encrypt(j0), y)
    }

    private fun gfMultiply(x: ByteArray, y: ByteArray): ByteArray {
        var z = ByteArray(16)
        var v = y.copyOf()
        for (i in 0 until 128) {
            val bit = (x[i / 8].toInt() ushr (7 - (i % 8))) and 1
            if (bit == 1) z = xor(z, v)
            val lsb = v[15].toInt() and 1
            for (j in 15 downTo 1) v[j] = ((v[j].toInt() ushr 1) or ((v[j - 1].toInt() and 1) shl 7)).toByte()
            v[0] = (v[0].toInt() ushr 1).toByte()
            if (lsb == 1) v[0] = (v[0].toInt() xor 0xe1).toByte()
        }
        return z
    }

    private fun xor(a: ByteArray, b: ByteArray): ByteArray = ByteArray(a.size) { a[it] xor b[it] }

    private class Aes256(key: ByteArray) {
        private val roundKeys = expandKey(key)

        fun encrypt(input: ByteArray): ByteArray {
            var state = input.copyOf()
            addRoundKey(state, roundKeys, 0)
            for (round in 1 until 14) {
                subBytes(state); shiftRows(state); mixColumns(state); addRoundKey(state, roundKeys, round)
            }
            subBytes(state); shiftRows(state); addRoundKey(state, roundKeys, 14)
            return state
        }

        private fun addRoundKey(state: ByteArray, keys: ByteArray, round: Int) {
            for (i in 0 until 16) state[i] = state[i] xor keys[round * 16 + i]
        }

        private fun subBytes(s: ByteArray) { for (i in s.indices) s[i] = sbox[s[i].toInt() and 255].toByte() }
        private fun shiftRows(s: ByteArray) {
            val t = s.copyOf()
            for (row in 0 until 4) for (col in 0 until 4) s[row + 4 * col] = t[row + 4 * ((col + row) % 4)]
        }
        private fun mixColumns(s: ByteArray) {
            for (c in 0 until 4) {
                val i = c * 4; val a = s[i].toInt() and 255; val b = s[i + 1].toInt() and 255; val d = s[i + 2].toInt() and 255; val e = s[i + 3].toInt() and 255
                s[i] = (gmul2(a) xor gmul3(b) xor d xor e).toByte(); s[i + 1] = (a xor gmul2(b) xor gmul3(d) xor e).toByte(); s[i + 2] = (a xor b xor gmul2(d) xor gmul3(e)).toByte(); s[i + 3] = (gmul3(a) xor b xor d xor gmul2(e)).toByte()
            }
        }
        private fun gmul2(x: Int) = ((x shl 1) xor (if (x and 128 != 0) 0x11b else 0)) and 255
        private fun gmul3(x: Int) = gmul2(x) xor x
    }

    private fun expandKey(key: ByteArray): ByteArray {
        require(key.size == 32)
        val out = ByteArray(240); key.copyInto(out)
        var bytes = 32; var rcon = 1
        val temp = ByteArray(4)
        while (bytes < 240) {
            for (i in 0 until 4) temp[i] = out[bytes - 4 + i]
            if (bytes % 32 == 0) {
                val t = temp[0]; temp[0] = temp[1]; temp[1] = temp[2]; temp[2] = temp[3]; temp[3] = t
                for (i in 0 until 4) temp[i] = sbox[temp[i].toInt() and 255].toByte()
                temp[0] = (temp[0].toInt() xor rcon).toByte(); rcon = if (rcon shl 1 > 255) (rcon shl 1) xor 0x11b else rcon shl 1
            } else if (bytes % 32 == 16) for (i in 0 until 4) temp[i] = sbox[temp[i].toInt() and 255].toByte()
            for (i in 0 until 4) { out[bytes] = (out[bytes - 32] xor temp[i]).also { temp[i] = it }; bytes++ }
        }
        return out
    }

    private val sbox = intArrayOf(
        0x63,0x7c,0x77,0x7b,0xf2,0x6b,0x6f,0xc5,0x30,0x01,0x67,0x2b,0xfe,0xd7,0xab,0x76,0xca,0x82,0xc9,0x7d,0xfa,0x59,0x47,0xf0,0xad,0xd4,0xa2,0xaf,0x9c,0xa4,0x72,0xc0,0xb7,0xfd,0x93,0x26,0x36,0x3f,0xf7,0xcc,0x34,0xa5,0xe5,0xf1,0x71,0xd8,0x31,0x15,0x04,0xc7,0x23,0xc3,0x18,0x96,0x05,0x9a,0x07,0x12,0x80,0xe2,0xeb,0x27,0xb2,0x75,0x09,0x83,0x2c,0x1a,0x1b,0x6e,0x5a,0xa0,0x52,0x3b,0xd6,0xb3,0x29,0xe3,0x2f,0x84,0x53,0xd1,0x00,0xed,0x20,0xfc,0xb1,0x5b,0x6a,0xcb,0xbe,0x39,0x4a,0x4c,0x58,0xcf,0xd0,0xef,0xaa,0xfb,0x43,0x4d,0x33,0x85,0x45,0xf9,0x02,0x7f,0x50,0x3c,0x9f,0xa8,0x51,0xa3,0x40,0x8f,0x92,0x9d,0x38,0xf5,0xbc,0xb6,0xda,0x21,0x10,0xff,0xf3,0xd2,0xcd,0x0c,0x13,0xec,0x5f,0x97,0x44,0x17,0xc4,0xa7,0x7e,0x3d,0x64,0x5d,0x19,0x73,0x60,0x81,0x4f,0xdc,0x22,0x2a,0x90,0x88,0x46,0xee,0xb8,0x14,0xde,0x5e,0x0b,0xdb,0xe0,0x32,0x3a,0x0a,0x49,0x06,0x24,0x5c,0xc2,0xd3,0xac,0x62,0x91,0x95,0xe4,0x79,0xe7,0xc8,0x37,0x6d,0x8d,0xd5,0x4e,0xa9,0x6c,0x56,0xf4,0xea,0x65,0x7a,0xae,0x08,0xba,0x78,0x25,0x2e,0x1c,0xa6,0xb4,0xc6,0xe8,0xdd,0x74,0x1f,0x4b,0xbd,0x8b,0x8a,0x70,0x3e,0xb5,0x66,0x48,0x03,0xf6,0x0e,0x61,0x35,0x57,0xb9,0x86,0xc1,0x1d,0x9e,0xe1,0xf8,0x98,0x11,0x69,0xd9,0x8e,0x94,0x9b,0x1e,0x87,0xe9,0xce,0x55,0x28,0xdf,0x8c,0xa1,0x89,0x0d,0xbf,0xe6,0x42,0x68,0x41,0x99,0x2d,0x0f,0xb0,0x54,0xbb,0x16,
    )
}

private fun encodeBase64Url(bytes: ByteArray): String {
    val alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_"
    val out = StringBuilder((bytes.size * 4 + 2) / 3); var i = 0
    while (i < bytes.size) { val a = bytes[i++].toInt() and 255; val b = if (i < bytes.size) bytes[i++].toInt() and 255 else -1; val c = if (i < bytes.size) bytes[i++].toInt() and 255 else -1; out.append(alphabet[a ushr 2]); out.append(alphabet[((a and 3) shl 4) or if (b >= 0) b ushr 4 else 0]); if (b >= 0) out.append(alphabet[((b and 15) shl 2) or if (c >= 0) c ushr 6 else 0]); if (c >= 0) out.append(alphabet[c and 63]) }
    return out.toString()
}

private fun decodeBase64Url(value: String): ByteArray {
    val alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_"; val clean = value.trim().trimEnd('='); val out = ArrayList<Byte>(); var buffer = 0; var bits = 0
    for (ch in clean) { val n = alphabet.indexOf(ch); require(n >= 0) { "Invalid base64url" }; buffer = (buffer shl 6) or n; bits += 6; if (bits >= 8) { bits -= 8; out.add((buffer ushr bits).toByte()); buffer = buffer and ((1 shl bits) - 1) } }
    return out.toByteArray()
}
