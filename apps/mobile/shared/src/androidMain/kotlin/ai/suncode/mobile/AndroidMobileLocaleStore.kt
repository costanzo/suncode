package ai.suncode.mobile

import android.content.Context

class AndroidMobileLocaleStore(context: Context) : MobileLocaleStore {
    private val preferences = context.applicationContext.getSharedPreferences("suncode.mobile.preferences", Context.MODE_PRIVATE)
    override suspend fun read(): String? = preferences.getString(KEY_LOCALE, null)
    override suspend fun write(locale: String) { preferences.edit().putString(KEY_LOCALE, locale).apply() }

    private companion object { const val KEY_LOCALE = "ui_locale" }
}
