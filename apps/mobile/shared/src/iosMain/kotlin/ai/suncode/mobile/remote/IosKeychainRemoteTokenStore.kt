@file:OptIn(kotlinx.cinterop.ExperimentalForeignApi::class)

package ai.suncode.mobile.remote

import ai.suncode.mobile.remote.RemoteHostCredentials
import ai.suncode.mobile.remote.RemoteHostCredentialStore
import kotlinx.cinterop.alloc
import kotlinx.cinterop.addressOf
import kotlinx.cinterop.memScoped
import kotlinx.cinterop.ptr
import kotlinx.cinterop.reinterpret
import kotlinx.cinterop.usePinned
import kotlinx.cinterop.CPointer
import kotlinx.cinterop.UByteVar
import kotlinx.cinterop.value
import kotlinx.cinterop.readBytes
import kotlinx.serialization.json.Json
import platform.CoreFoundation.CFDataCreate
import platform.CoreFoundation.CFDataGetBytePtr
import platform.CoreFoundation.CFDataGetLength
import platform.CoreFoundation.CFDataRef
import platform.CoreFoundation.CFDictionaryCreateMutable
import platform.CoreFoundation.CFDictionarySetValue
import platform.CoreFoundation.CFRelease
import platform.CoreFoundation.CFStringCreateWithCString
import platform.CoreFoundation.CFTypeRefVar
import platform.CoreFoundation.kCFBooleanTrue
import platform.CoreFoundation.kCFStringEncodingUTF8
import platform.CoreFoundation.kCFTypeDictionaryKeyCallBacks
import platform.CoreFoundation.kCFTypeDictionaryValueCallBacks
import platform.Security.SecItemAdd
import platform.Security.SecItemCopyMatching
import platform.Security.SecItemDelete
import platform.Security.errSecItemNotFound
import platform.Security.errSecSuccess
import platform.Security.kSecAttrAccount
import platform.Security.kSecAttrAccessible
import platform.Security.kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly
import platform.Security.kSecAttrService
import platform.Security.kSecClass
import platform.Security.kSecClassGenericPassword
import platform.Security.kSecMatchLimit
import platform.Security.kSecMatchLimitOne
import platform.Security.kSecReturnData
import platform.Security.kSecValueData

/** Stores the current device's opaque Remote Control credential in iOS Keychain. */
class IosKeychainRemoteTokenStore : RemoteTokenStore {
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
        val bytes = json.encodeToString(RemoteHostCredentialStore.serializer(), RemoteHostCredentialStore(values)).encodeToByteArray()
        val data = bytes.usePinned { CFDataCreate(null, it.addressOf(0).reinterpret<UByteVar>(), bytes.size.toLong()) }
            ?: error("Unable to encode Remote Control credential")
        val query = baseQuery()
        try {
            CFDictionarySetValue(query, kSecValueData, data)
            CFDictionarySetValue(query, kSecAttrAccessible, kSecAttrAccessibleAfterFirstUnlockThisDeviceOnly)
            deleteStored()
            check(SecItemAdd(query, null) == errSecSuccess) { "Unable to save Remote Control credential" }
        } finally {
            CFRelease(data)
            CFRelease(query)
        }
    }

    private fun deleteStored() {
        val query = baseQuery()
        try {
            val status = SecItemDelete(query)
            check(status == errSecSuccess || status == errSecItemNotFound) { "Unable to clear Remote Control credential" }
        } finally {
            CFRelease(query)
        }
    }

    private fun read(): Map<String, RemoteHostCredentials> = memScoped {
        val query = baseQuery()
        try {
            CFDictionarySetValue(query, kSecReturnData, kCFBooleanTrue)
            CFDictionarySetValue(query, kSecMatchLimit, kSecMatchLimitOne)
            val result = alloc<CFTypeRefVar>()
            when (SecItemCopyMatching(query, result.ptr)) {
                errSecItemNotFound -> emptyMap()
                errSecSuccess -> {
                    val data = result.value as? CFDataRef
                        ?: error("Invalid Remote Control credential")
                    try {
                        val size = CFDataGetLength(data).toInt()
                        val pointer: CPointer<UByteVar> = CFDataGetBytePtr(data)
                            ?: error("Invalid Remote Control credential")
                        val bytes = pointer.readBytes(size)
                        json.decodeFromString<RemoteHostCredentialStore>(bytes.decodeToString()).hosts
                    } finally {
                        CFRelease(data)
                    }
                }
                else -> error("Unable to read Remote Control credential")
            }
        } finally {
            CFRelease(query)
        }
    }

    private fun baseQuery() = CFDictionaryCreateMutable(
        null, 0, kCFTypeDictionaryKeyCallBacks.ptr, kCFTypeDictionaryValueCallBacks.ptr,
    )!!.also { query ->
        CFDictionarySetValue(query, kSecClass, kSecClassGenericPassword)
        setString(query, kSecAttrService, SERVICE)
        setString(query, kSecAttrAccount, ACCOUNT)
    }

    private fun setString(query: platform.CoreFoundation.CFMutableDictionaryRef?, key: platform.CoreFoundation.CFStringRef?, value: String) {
        val string = CFStringCreateWithCString(null, value, kCFStringEncodingUTF8)!!
        try {
            CFDictionarySetValue(query, key, string)
        } finally {
            CFRelease(string)
        }
    }

    private companion object {
        const val SERVICE = "ai.suncode.mobile.remote-control"
        const val ACCOUNT = "device-credential-v1"
    }
}
