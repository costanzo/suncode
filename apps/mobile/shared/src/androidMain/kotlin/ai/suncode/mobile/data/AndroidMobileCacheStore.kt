package ai.suncode.mobile.data

import android.content.Context

/** Stores the last usable Mobile projection in app-private SharedPreferences. */
class AndroidMobileCacheStore(context: Context) : MobileCacheStore {
    private val preferences = context.applicationContext.getSharedPreferences(PREFERENCES, Context.MODE_PRIVATE)

    override suspend fun read(): String? = preferences.getString(KEY, null)

    override suspend fun write(value: String) {
        preferences.edit().putString(KEY, value).apply()
    }

    override suspend fun clear() {
        preferences.edit().remove(KEY).apply()
    }

    private companion object {
        const val PREFERENCES = "suncode.mobile.projection"
        const val KEY = "snapshot_v1"
    }
}
