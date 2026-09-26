package ai.suncode.mobile.data

import platform.Foundation.NSUserDefaults

/** Stores the last usable Mobile projection in the app's standard UserDefaults domain. */
class IosMobileCacheStore : MobileCacheStore {
    private val defaults = NSUserDefaults.standardUserDefaults

    override suspend fun read(): String? = defaults.stringForKey(KEY)

    override suspend fun write(value: String) {
        defaults.setObject(value, forKey = KEY)
    }

    override suspend fun clear() {
        defaults.removeObjectForKey(KEY)
    }

    private companion object {
        const val KEY = "suncode.mobile.projection.v1"
    }
}
