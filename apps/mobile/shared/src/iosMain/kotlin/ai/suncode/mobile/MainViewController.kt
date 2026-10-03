package ai.suncode.mobile

import ai.suncode.mobile.data.IosMobileCacheStore
import ai.suncode.mobile.data.MobileRepository
import ai.suncode.mobile.data.RemoteMobileRepository
import ai.suncode.mobile.remote.IosKeychainRemoteTokenStore
import ai.suncode.mobile.remote.KtorRemoteControlClient
import androidx.compose.ui.window.ComposeUIViewController

fun MainViewController(onScanPairing: (((String) -> Unit) -> Unit)? = null) = createRepository().let { repository ->
    ComposeUIViewController { App(repository, onScanPairing = onScanPairing, localeStore = IosMobileLocaleStore()) }
}

private fun createRepository(): MobileRepository {
    val client = KtorRemoteControlClient(IosKeychainRemoteTokenStore())
    return RemoteMobileRepository(client, cacheStore = IosMobileCacheStore())
}
