package ai.suncode.mobile

import android.os.Bundle
import android.content.Intent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.activity.ComponentActivity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.runtime.Composable
import androidx.compose.ui.tooling.preview.Preview
import ai.suncode.mobile.data.FakeMobileRepository
import ai.suncode.mobile.data.AndroidMobileCacheStore
import ai.suncode.mobile.data.MobileRepository
import ai.suncode.mobile.data.RemoteMobileRepository
import ai.suncode.mobile.remote.AndroidKeystoreRemoteTokenStore
import ai.suncode.mobile.remote.KtorRemoteControlClient

class MainActivity : ComponentActivity() {
    private var client: KtorRemoteControlClient? = null

    override fun onCreate(savedInstanceState: Bundle?) {
        enableEdgeToEdge()
        super.onCreate(savedInstanceState)
        val repository = createRepository()

        setContent {
            val scanner = rememberLauncherForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->
                val payload = result.data?.getStringExtra(QrScannerActivity.EXTRA_PAYLOAD)
                if (result.resultCode == RESULT_OK && !payload.isNullOrBlank()) pendingScanCallback?.invoke(payload)
                pendingScanCallback = null
            }
            App(repository) { onResult ->
                pendingScanCallback = onResult
                scanner.launch(Intent(this, QrScannerActivity::class.java))
            }
        }
    }

    private var pendingScanCallback: ((String) -> Unit)? = null

    override fun onDestroy() {
        client?.close()
        client = null
        super.onDestroy()
    }

    private fun createRepository(): MobileRepository {
        val baseUrl = BuildConfig.REMOTE_CONTROL_BASE_URL.trim()
        if (baseUrl.isBlank()) return FakeMobileRepository()
        val remoteClient = KtorRemoteControlClient(
            baseUrl = baseUrl,
            tokenStore = AndroidKeystoreRemoteTokenStore(applicationContext),
        )
        client = remoteClient
        return RemoteMobileRepository(remoteClient, cacheStore = AndroidMobileCacheStore(applicationContext))
    }
}

@Preview
@Composable
fun AppAndroidPreview() {
    App()
}
