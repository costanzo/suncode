package ai.suncode.mobile

import ai.suncode.mobile.data.FakeMobileRepository
import ai.suncode.mobile.data.IosMobileCacheStore
import ai.suncode.mobile.data.MobileRepository
import ai.suncode.mobile.data.RemoteMobileRepository
import ai.suncode.mobile.remote.IosKeychainRemoteTokenStore
import ai.suncode.mobile.remote.KtorRemoteControlClient
import androidx.compose.ui.window.ComposeUIViewController
import platform.Foundation.NSBundle

fun MainViewController(onScanPairing: (((String) -> Unit) -> Unit)? = null) = createRepository().let { repository ->
    ComposeUIViewController { App(repository, onScanPairing = onScanPairing) }
}

private fun createRepository(): MobileRepository {
    val baseUrl = NSBundle.mainBundle.objectForInfoDictionaryKey("RemoteControlBaseURL") as? String
    if (baseUrl.isNullOrBlank()) return FakeMobileRepository()
    val client = KtorRemoteControlClient(baseUrl, IosKeychainRemoteTokenStore())
    return RemoteMobileRepository(client, cacheStore = IosMobileCacheStore())
}
