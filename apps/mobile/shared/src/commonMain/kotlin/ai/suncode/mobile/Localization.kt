package ai.suncode.mobile

import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.compositionLocalOf

enum class MobileLocale(val code: String, val nativeName: String) {
    ENGLISH("en-US", "English"),
    SIMPLIFIED_CHINESE("zh-CN", "简体中文");

    companion object {
        fun fromCode(value: String?): MobileLocale = entries.firstOrNull { it.code.equals(value, ignoreCase = true) } ?: ENGLISH
    }
}

interface MobileLocaleStore {
    suspend fun read(): String?
    suspend fun write(locale: String)
}

class InMemoryMobileLocaleStore(initial: String? = null) : MobileLocaleStore {
    private var value: String? = initial
    override suspend fun read(): String? = value
    override suspend fun write(locale: String) { value = locale }
}

@Immutable
class MobileStrings(val locale: MobileLocale) {
    private val values = if (locale == MobileLocale.SIMPLIFIED_CHINESE) ZH else EN

    fun get(key: String, vararg args: String): String {
        var result = values[key] ?: EN[key] ?: key
        args.forEach { result = result.replaceFirst("%s", it) }
        return result
    }
}

val LocalMobileStrings = compositionLocalOf { MobileStrings(MobileLocale.ENGLISH) }

@Composable
fun mobileString(key: String, vararg args: String): String = LocalMobileStrings.current.get(key, *args)

private val EN = mapOf(
    "sessions" to "Sessions", "hosts" to "Hosts", "settings" to "Settings", "appearance" to "Appearance", "theme" to "Theme",
    "language" to "Language", "interfaceLanguage" to "Interface language", "security" to "Security", "pairedDevices" to "Paired devices",
    "offlineCache" to "Offline cache", "about" to "About", "appVersion" to "App version", "protocol" to "Protocol", "selectSession" to "Select a session",
    "allHosts" to "All hosts", "allProjects" to "All projects", "updatedJustNow" to "Updated just now", "newSession" to "New session",
    "back" to "Back", "more" to "More", "messageSunCode" to "Message SunCode…", "sendMessage" to "Send message", "scanQr" to "Scan QR code",
    "projects" to "Projects", "projectCount" to "%s projects", "activeSessions" to "%s active sessions", "remoteServer" to "Remote Server", "endpoint" to "Endpoint",
    "allowOnce" to "Allow once", "deny" to "Deny", "sendAnswer" to "Send answer", "pairNewDesktop" to "Pair a new Desktop",
    "pairingHint" to "Enter the one-time pairing payload shown by SunCode Desktop.", "enterPairingPayload" to "Enter pairing payload",
    "confirmPairing" to "Confirm pairing", "pairingPayloadTitle" to "Enter pairing payload", "exchangePayload" to "Exchange this one-time payload with Remote Server?",
    "scanOrPaste" to "Scan the QR code or paste the payload from SunCode Desktop.", "openCamera" to "Open camera", "pairingPayload" to "Pairing payload",
    "pairing" to "Pairing…", "continue" to "Continue", "cancel" to "Cancel", "pairingFailed" to "Pairing failed. Check the payload and try again.",
    "pairingRequired" to "Pairing payload is required", "noHost" to "No Host", "noProject" to "No Project", "sessionTitle" to "Session title",
    "firstMessage" to "First message", "createSession" to "Create session", "light" to "Light", "dark" to "Dark", "system" to "System",
    "connectedTo" to "Connected to %s", "connectionUpdated" to "Updated just now", "connected" to "Connected", "connecting" to "Connecting",
    "degraded" to "Degraded", "offline" to "Offline", "unauthorized" to "Unauthorized", "incompatible" to "Incompatible", "idle" to "Idle",
    "running" to "Running", "waitingApproval" to "Waiting for approval", "waitingAnswer" to "Waiting for answer", "failed" to "Failed",
    "languageDialogTitle" to "Language", "english" to "English", "simplifiedChinese" to "简体中文", "copy" to "Copy"
)

private val ZH = mapOf(
    "sessions" to "会话", "hosts" to "主机", "settings" to "设置", "appearance" to "外观", "theme" to "主题",
    "language" to "语言", "interfaceLanguage" to "界面语言", "security" to "安全", "pairedDevices" to "已配对设备",
    "offlineCache" to "离线缓存", "about" to "关于", "appVersion" to "应用版本", "protocol" to "协议", "selectSession" to "选择一个会话",
    "allHosts" to "所有主机", "allProjects" to "所有项目", "updatedJustNow" to "刚刚更新", "newSession" to "新建会话",
    "back" to "返回", "more" to "更多", "messageSunCode" to "发送消息给 SunCode…", "sendMessage" to "发送消息", "scanQr" to "扫描二维码",
    "projects" to "项目", "projectCount" to "%s 个项目", "activeSessions" to "%s 个活动会话", "remoteServer" to "远程服务器", "endpoint" to "端点",
    "allowOnce" to "允许一次", "deny" to "拒绝", "sendAnswer" to "发送回答", "pairNewDesktop" to "配对新的桌面客户端",
    "pairingHint" to "输入 SunCode Desktop 显示的一次性配对内容。", "enterPairingPayload" to "输入配对内容",
    "confirmPairing" to "确认配对", "pairingPayloadTitle" to "输入配对内容", "exchangePayload" to "要将此一次性内容交给远程服务器吗？",
    "scanOrPaste" to "扫描二维码，或粘贴 SunCode Desktop 提供的配对内容。", "openCamera" to "打开相机", "pairingPayload" to "配对内容",
    "pairing" to "配对中…", "continue" to "继续", "cancel" to "取消", "pairingFailed" to "配对失败。请检查配对内容后重试。",
    "pairingRequired" to "需要配对内容", "noHost" to "没有主机", "noProject" to "没有项目", "sessionTitle" to "会话标题",
    "firstMessage" to "第一条消息", "createSession" to "创建会话", "light" to "浅色", "dark" to "深色", "system" to "跟随系统",
    "connectedTo" to "已连接到 %s", "connectionUpdated" to "刚刚更新", "connected" to "已连接", "connecting" to "连接中",
    "degraded" to "连接降级", "offline" to "离线", "unauthorized" to "未授权", "incompatible" to "不兼容", "idle" to "空闲",
    "running" to "运行中", "waitingApproval" to "等待批准", "waitingAnswer" to "等待回答", "failed" to "失败",
    "languageDialogTitle" to "语言", "english" to "English", "simplifiedChinese" to "简体中文", "copy" to "复制"
)

internal val mobileEnglishLocalizationKeys: Set<String> = EN.keys
internal val mobileChineseLocalizationKeys: Set<String> = ZH.keys
