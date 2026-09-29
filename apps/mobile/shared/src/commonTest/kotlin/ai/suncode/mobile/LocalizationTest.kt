package ai.suncode.mobile

import kotlinx.coroutines.runBlocking
import kotlin.test.Test
import kotlin.test.assertEquals

class LocalizationTest {
    @Test
    fun unsupportedLocaleFallsBackToEnglish() {
        assertEquals(MobileLocale.ENGLISH, MobileLocale.fromCode("fr-FR"))
        assertEquals(MobileLocale.SIMPLIFIED_CHINESE, MobileLocale.fromCode("zh-cn"))
    }

    @Test
    fun localeUsesStableCodesAndNativeNames() {
        assertEquals("en-US", MobileLocale.ENGLISH.code)
        assertEquals("English", MobileLocale.ENGLISH.nativeName)
        assertEquals("zh-CN", MobileLocale.SIMPLIFIED_CHINESE.code)
        assertEquals("简体中文", MobileLocale.SIMPLIFIED_CHINESE.nativeName)
    }

    @Test
    fun stringsChangeWithLocaleAndFormatArguments() {
        assertEquals("Connected to MacBook Pro", MobileStrings(MobileLocale.ENGLISH).get("connectedTo", "MacBook Pro"))
        assertEquals("已连接到 MacBook Pro", MobileStrings(MobileLocale.SIMPLIFIED_CHINESE).get("connectedTo", "MacBook Pro"))
        assertEquals("3 projects", MobileStrings(MobileLocale.ENGLISH).get("projectCount", "3"))
        assertEquals("3 个项目", MobileStrings(MobileLocale.SIMPLIFIED_CHINESE).get("projectCount", "3"))
    }

    @Test
    fun localeStorePersistsSelection() = runBlocking {
        val store = InMemoryMobileLocaleStore()
        store.write(MobileLocale.SIMPLIFIED_CHINESE.code)
        assertEquals("zh-CN", store.read())
    }

    @Test
    fun localeCatalogsHaveMatchingKeys() {
        assertEquals(mobileEnglishLocalizationKeys, mobileChineseLocalizationKeys)
    }
}
