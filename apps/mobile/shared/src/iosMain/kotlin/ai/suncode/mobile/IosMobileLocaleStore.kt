package ai.suncode.mobile

import platform.Foundation.NSUserDefaults

class IosMobileLocaleStore : MobileLocaleStore {
    private val defaults = NSUserDefaults.standardUserDefaults
    override suspend fun read(): String? = defaults.stringForKey(KEY_LOCALE)
    override suspend fun write(locale: String) { defaults.setObject(locale, forKey = KEY_LOCALE) }

    private companion object { const val KEY_LOCALE = "ui_locale" }
}
