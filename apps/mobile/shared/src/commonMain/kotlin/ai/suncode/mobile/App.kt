package ai.suncode.mobile

import ai.suncode.mobile.data.FakeMobileRepository
import ai.suncode.mobile.data.MobileRepository
import ai.suncode.mobile.domain.*
import ai.suncode.mobile.ui.theme.SunCodeTheme
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.*
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlinx.coroutines.launch

private data class AppState(
    val tab: AppTab = AppTab.SESSIONS,
    val session: Session? = null,
    val host: Host? = null,
    val theme: ThemePreference = ThemePreference.SYSTEM,
)

@Composable
fun App(
    repository: MobileRepository = FakeMobileRepository(),
    onScanPairing: (((String) -> Unit) -> Unit)? = null,
) {
    var state by remember { mutableStateOf(AppState()) }
    SunCodeTheme(state.theme) { MobileRoot(repository, onScanPairing, state) { state = it } }
}

@Composable
private fun MobileRoot(repository: MobileRepository, onScanPairing: (((String) -> Unit) -> Unit)?, state: AppState, setState: (AppState) -> Unit) {
    val sessions by repository.observeSessions().collectAsStateWithLifecycle(emptyList())
    val hosts by repository.observeHosts().collectAsStateWithLifecycle(emptyList())
    BoxWithConstraints(Modifier.fillMaxSize().safeDrawingPadding()) {
        if (maxWidth >= 700.dp) {
            TabletShell(repository, onScanPairing, state, sessions, hosts, setState)
        } else {
            Scaffold(bottomBar = { if (state.session == null && state.host == null) BottomNav(state.tab) { setState(state.copy(tab = it)) } }) { padding ->
                when {
                    state.session != null -> SessionDetail(repository, state.session, setState, Modifier.padding(padding))
                    state.host != null -> HostDetail(state.host, setState, Modifier.padding(padding))
                    state.tab == AppTab.SESSIONS -> SessionList(repository, sessions, hosts, setState, Modifier.padding(padding))
                    state.tab == AppTab.HOSTS -> HostList(repository, onScanPairing, hosts, setState, Modifier.padding(padding))
                    else -> Settings(state.theme, setState, Modifier.padding(padding))
                }
            }
        }
    }
}

@Composable
private fun TabletShell(repository: MobileRepository, onScanPairing: (((String) -> Unit) -> Unit)?, state: AppState, sessions: List<Session>, hosts: List<Host>, setState: (AppState) -> Unit) {
    Row(Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background)) {
        Column(Modifier.width(150.dp).fillMaxHeight().background(MaterialTheme.colorScheme.surface).padding(12.dp)) {
            Text("SunCode", fontWeight = FontWeight.Bold, modifier = Modifier.padding(8.dp))
            RailItem("Sessions", Icons.Default.Chat, AppTab.SESSIONS, state.tab, setState)
            RailItem("Hosts", Icons.Default.Devices, AppTab.HOSTS, state.tab, setState)
            RailItem("Settings", Icons.Default.Settings, AppTab.SETTINGS, state.tab, setState)
        }
        Column(Modifier.width(330.dp).fillMaxHeight().padding(16.dp)) {
            Text("Sessions", style = MaterialTheme.typography.headlineSmall)
            Spacer(Modifier.height(12.dp))
            LazyColumn(verticalArrangement = Arrangement.spacedBy(8.dp)) { items(sessions, key = { it.id }) { SessionRow(it) { setState(state.copy(session = it)) } } }
        }
        Box(Modifier.weight(1f).fillMaxHeight().padding(16.dp)) {
            when {
                state.session != null -> SessionDetail(repository, state.session, setState, Modifier.fillMaxSize())
                state.tab == AppTab.HOSTS -> HostList(repository, onScanPairing, hosts, setState, Modifier.fillMaxSize())
                state.tab == AppTab.SETTINGS -> Settings(state.theme, setState, Modifier.fillMaxSize())
                else -> Box(Modifier.fillMaxSize(), contentAlignment = Alignment.Center) { Text("Select a session", color = MaterialTheme.colorScheme.onSurfaceVariant) }
            }
        }
    }
}

@Composable
private fun SessionList(repository: MobileRepository, sessions: List<Session>, hosts: List<Host>, setState: (AppState) -> Unit, modifier: Modifier) {
    var hostFilter by remember { mutableStateOf("All hosts") }
    var projectFilter by remember { mutableStateOf("All projects") }
    var showNewSession by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()
    val projects = hosts.flatMap { it.projects }.distinctBy { it.id }
    val visible = sessions.filter { (hostFilter == "All hosts" || it.hostName == hostFilter) && (projectFilter == "All projects" || it.projectName == projectFilter) }
    Scaffold(modifier, topBar = { TopAppBar(title = { Text("Sessions") }, actions = { IconButton({ showNewSession = true }) { Icon(Icons.Default.Add, "New session") } }, colors = TopAppBarDefaults.topAppBarColors(containerColor = Color.Transparent)) }) { padding ->
        Column(Modifier.padding(padding).padding(horizontal = 16.dp)) {
            Row(Modifier.horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                FilterChip(hostFilter == "All hosts", { hostFilter = "All hosts" }, label = { Text("All hosts") })
                hosts.forEach { FilterChip(hostFilter == it.name, { hostFilter = it.name }, label = { Text(it.name) }) }
                FilterChip(projectFilter == "All projects", { projectFilter = "All projects" }, label = { Text("All projects") })
                projects.forEach { FilterChip(projectFilter == it.name, { projectFilter = it.name }, label = { Text(it.name) }) }
            }
            ConnectionBanner("Updated just now", HostConnectionState.CONNECTED)
            LazyColumn(verticalArrangement = Arrangement.spacedBy(8.dp), contentPadding = PaddingValues(bottom = 20.dp)) { items(visible, key = { it.id }) { SessionRow(it) { setState(AppState(session = it)) } } }
        }
    }
    if (showNewSession) {
        NewSessionDialog(
            hosts = hosts,
            onCreate = { title, message ->
                val host = hosts.firstOrNull() ?: return@NewSessionDialog
                val project = host.projects.firstOrNull() ?: return@NewSessionDialog
                scope.launch {
                    repository.createSession(host, project, title, message).onSuccess {
                        showNewSession = false
                    }
                }
            },
            onDismiss = { showNewSession = false },
        )
    }
}

@Composable
private fun SessionDetail(repository: MobileRepository, session: Session, setState: (AppState) -> Unit, modifier: Modifier) {
    var message by remember { mutableStateOf("") }
    val scope = rememberCoroutineScope()
    Column(modifier.fillMaxSize()) {
        TopAppBar(title = { Text(session.title, maxLines = 1, overflow = TextOverflow.Ellipsis) }, navigationIcon = { IconButton({ setState(AppState()) }) { Icon(Icons.Default.ArrowBack, "Back") } }, actions = { IconButton({}) { Icon(Icons.Default.MoreVert, "More") } }, colors = TopAppBarDefaults.topAppBarColors(containerColor = Color.Transparent))
        Column(Modifier.weight(1f).verticalScroll(rememberScrollState()).padding(horizontal = 16.dp)) {
            ConnectionBanner("Connected to ${session.hostName}", HostConnectionState.CONNECTED)
            session.messages.forEach { MessageBubble(it) }
            session.pendingApproval?.let { approval ->
                ApprovalCard(approval) { action -> scope.launch { repository.resolveApproval(session.id, approval.id, action, approval.revision) } }
            }
            session.pendingQuestion?.let { question ->
                QuestionCard(question) { answer -> scope.launch { repository.answerQuestion(session.id, answer) } }
            }
        }
        Row(Modifier.fillMaxWidth().padding(12.dp), verticalAlignment = Alignment.CenterVertically) {
            OutlinedTextField(message, { message = it }, Modifier.weight(1f), placeholder = { Text("Message SunCode…") }, maxLines = 4)
            IconButton({
                if (message.isNotBlank()) {
                    scope.launch { repository.sendMessage(session.id, message); message = "" }
                }
            }) { Icon(Icons.Default.Send, "Send message") }
        }
    }
}

@Composable
private fun HostList(repository: MobileRepository, onScanPairing: (((String) -> Unit) -> Unit)?, hosts: List<Host>, setState: (AppState) -> Unit, modifier: Modifier) {
    var showPairing by remember { mutableStateOf(false) }
    var pairingError by remember { mutableStateOf<String?>(null) }
    var pairingInProgress by remember { mutableStateOf(false) }
    val scope = rememberCoroutineScope()
    Scaffold(modifier, topBar = { TopAppBar(title = { Text("Hosts") }, actions = { IconButton({ pairingError = null; showPairing = true }) { Icon(Icons.Default.QrCodeScanner, "Scan QR code") } }, colors = TopAppBarDefaults.topAppBarColors(containerColor = Color.Transparent)) }) { padding ->
        Column(Modifier.padding(padding).padding(horizontal = 16.dp)) {
            LazyColumn(verticalArrangement = Arrangement.spacedBy(8.dp)) { items(hosts, key = { it.id }) { HostRow(it) { setState(AppState(tab = AppTab.HOSTS, host = it)) } } }
            PairingCard { showPairing = true }
        }
    }
    if (showPairing) PairingDialog(
        onConfirm = { payload ->
            pairingInProgress = true
            pairingError = null
            scope.launch {
                repository.pairHost(payload).fold(
                    onSuccess = { showPairing = false },
                    onFailure = { pairingError = it.message ?: "Pairing failed. Check the payload and try again." },
                )
                pairingInProgress = false
            }
        },
        onScan = onScanPairing,
        errorText = pairingError,
        confirming = pairingInProgress,
        onDismiss = { showPairing = false },
    )
}

@Composable
private fun HostDetail(host: Host, setState: (AppState) -> Unit, modifier: Modifier) {
    Column(modifier.fillMaxSize()) {
        TopAppBar(title = { Text(host.name) }, navigationIcon = { IconButton({ setState(AppState(tab = AppTab.HOSTS)) }) { Icon(Icons.Default.ArrowBack, "Back") } }, colors = TopAppBarDefaults.topAppBarColors(containerColor = Color.Transparent))
        Column(Modifier.verticalScroll(rememberScrollState()).padding(horizontal = 16.dp)) {
            HostIdentity(host)
            Spacer(Modifier.height(18.dp))
            Text("Projects", style = MaterialTheme.typography.titleMedium, fontWeight = FontWeight.Bold)
            host.projects.forEach { ProjectRow(it) }
        }
    }
}

@Composable
private fun Settings(theme: ThemePreference, setState: (AppState) -> Unit, modifier: Modifier) {
    var showTheme by remember { mutableStateOf(false) }
    Scaffold(modifier, topBar = { TopAppBar(title = { Text("Settings") }, colors = TopAppBarDefaults.topAppBarColors(containerColor = Color.Transparent)) }) { padding ->
        Column(Modifier.padding(padding).padding(16.dp).verticalScroll(rememberScrollState())) {
            SettingsGroup("Appearance") { SettingRow(Icons.Default.DarkMode, "Theme", theme.displayName()) { showTheme = true } }
            SettingsGroup("Security") { SettingRow(Icons.Default.Devices, "Paired devices", "1") {}; SettingRow(Icons.Default.Cached, "Offline cache", "128 MB") {} }
            SettingsGroup("About") { SettingRow(Icons.Default.Settings, "App version", "0.1.0") {}; SettingRow(Icons.Default.Link, "Protocol", "remote.v1") {} }
        }
    }
    if (showTheme) ThemeDialog(theme, { setState(AppState(tab = AppTab.SETTINGS, theme = it)); showTheme = false }, { showTheme = false })
}

@Composable private fun BottomNav(selected: AppTab, onSelect: (AppTab) -> Unit) { NavigationBar { NavigationBarItem(selected == AppTab.SESSIONS, { onSelect(AppTab.SESSIONS) }, { Icon(Icons.Default.Chat, null) }, label = { Text("Sessions") }); NavigationBarItem(selected == AppTab.HOSTS, { onSelect(AppTab.HOSTS) }, { Icon(Icons.Default.Devices, null) }, label = { Text("Hosts") }); NavigationBarItem(selected == AppTab.SETTINGS, { onSelect(AppTab.SETTINGS) }, { Icon(Icons.Default.Settings, null) }, label = { Text("Settings") }) } }
@Composable private fun RailItem(label: String, icon: ImageVector, tab: AppTab, selected: AppTab, setState: (AppState) -> Unit) { Surface(color = if (tab == selected) MaterialTheme.colorScheme.primaryContainer else Color.Transparent, shape = RoundedCornerShape(10.dp), modifier = Modifier.fillMaxWidth().clickable { setState(AppState(tab = tab)) }) { Row(Modifier.padding(10.dp), horizontalArrangement = Arrangement.spacedBy(8.dp)) { Icon(icon, null, Modifier.size(18.dp)); Text(label, fontSize = 12.sp) } } }
@Composable private fun SessionRow(session: Session, onClick: () -> Unit) { Card(onClick = onClick, border = BorderStroke(1.dp, MaterialTheme.colorScheme.outline), modifier = Modifier.fillMaxWidth()) { Row(Modifier.padding(14.dp)) { Column(Modifier.weight(1f)) { Row(verticalAlignment = Alignment.CenterVertically) { StatusDot(session.state); Text(session.title, fontWeight = FontWeight.SemiBold, modifier = Modifier.padding(start = 7.dp)) }; Text(session.preview, color = MaterialTheme.colorScheme.onSurfaceVariant, fontSize = 12.sp, maxLines = 1, overflow = TextOverflow.Ellipsis, modifier = Modifier.padding(top = 5.dp)); Text("${session.hostName} · ${session.projectName}", color = MaterialTheme.colorScheme.onSurfaceVariant, fontSize = 11.sp, modifier = Modifier.padding(top = 4.dp)) }; Column(horizontalAlignment = Alignment.End) { Text(session.updatedLabel, fontSize = 10.sp, color = MaterialTheme.colorScheme.onSurfaceVariant); Text(session.state.displayName(), fontSize = 10.sp, color = sessionColor(session.state), modifier = Modifier.padding(top = 7.dp)) } } } }
@Composable private fun HostRow(host: Host, onClick: () -> Unit) { Card(onClick = onClick, border = BorderStroke(1.dp, MaterialTheme.colorScheme.outline), modifier = Modifier.fillMaxWidth()) { Row(Modifier.padding(14.dp), verticalAlignment = Alignment.CenterVertically) { Icon(Icons.Default.Devices, null, tint = MaterialTheme.colorScheme.secondary, modifier = Modifier.size(26.dp)); Column(Modifier.weight(1f).padding(start = 10.dp)) { Text(host.name, fontWeight = FontWeight.SemiBold); Text("${host.endpoint} · ${host.projects.size} projects", fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) }; Text(host.state.displayName(), fontSize = 10.sp, color = hostColor(host.state)) } } }
@Composable private fun HostIdentity(host: Host) { Card(border = BorderStroke(1.dp, MaterialTheme.colorScheme.outline), modifier = Modifier.fillMaxWidth()) { Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) { Row(verticalAlignment = Alignment.CenterVertically) { Icon(Icons.Default.Devices, null, tint = MaterialTheme.colorScheme.secondary, modifier = Modifier.size(28.dp)); Column(Modifier.weight(1f).padding(start = 10.dp)) { Text(host.name, fontWeight = FontWeight.Bold); Text(host.state.displayName(), fontSize = 11.sp, color = hostColor(host.state)) } }; DetailLine("Remote Server", "relay.suncode.dev"); DetailLine("Endpoint", host.endpoint); DetailLine("Protocol", "remote.v1") } } }
@Composable private fun ProjectRow(project: Project) { Row(Modifier.fillMaxWidth().padding(vertical = 12.dp), verticalAlignment = Alignment.CenterVertically) { Icon(Icons.Default.Folder, null, tint = MaterialTheme.colorScheme.secondary); Column(Modifier.weight(1f).padding(start = 10.dp)) { Text(project.name, fontWeight = FontWeight.SemiBold); Text("${project.activeSessions} active sessions", fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) }; Icon(Icons.Default.ChevronRight, null, Modifier.size(18.dp)) }; HorizontalDivider() }
@Composable private fun MessageBubble(message: Message) { Row(Modifier.fillMaxWidth(), horizontalArrangement = if (message.author == MessageAuthor.USER) Arrangement.End else Arrangement.Start) { Surface(color = if (message.author == MessageAuthor.USER) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.surface, contentColor = if (message.author == MessageAuthor.USER) MaterialTheme.colorScheme.onPrimary else MaterialTheme.colorScheme.onSurface, shape = RoundedCornerShape(14.dp), modifier = Modifier.padding(vertical = 6.dp).fillMaxWidth(.9f)) { Text(message.body, Modifier.padding(13.dp), fontSize = 13.sp) } } }
@Composable
private fun ApprovalCard(approval: PendingApproval, onResolve: (String) -> Unit) {
    Card(colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.tertiaryContainer), modifier = Modifier.fillMaxWidth().padding(vertical = 10.dp)) {
        Column(Modifier.padding(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(approval.summary, fontWeight = FontWeight.SemiBold)
            approval.detail?.let { Text(it, fontSize = 11.sp) }
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Button({ onResolve("allow_once") }) { Text("Allow once") }
                OutlinedButton({ onResolve("deny") }) { Text("Deny") }
            }
        }
    }
}

@Composable
private fun QuestionCard(question: PendingQuestion, onAnswer: (String) -> Unit) {
    var selected by remember(question.id) { mutableStateOf(question.options.firstOrNull().orEmpty()) }
    Card(colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.tertiaryContainer), modifier = Modifier.fillMaxWidth().padding(vertical = 10.dp)) {
        Column(Modifier.padding(12.dp), verticalArrangement = Arrangement.spacedBy(5.dp)) {
            Text(question.prompt, fontWeight = FontWeight.SemiBold)
            question.options.forEach { option ->
                Text(option, modifier = Modifier.fillMaxWidth().clickable { selected = option }.padding(10.dp), color = if (option == selected) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurface)
            }
            Button({ if (selected.isNotBlank()) onAnswer(selected) }, enabled = selected.isNotBlank()) { Text("Send answer") }
        }
    }
}
@Composable private fun PairingCard(onClick: () -> Unit) { Card(colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant), modifier = Modifier.fillMaxWidth().padding(top = 12.dp)) { Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) { Text("Pair a new Desktop", fontWeight = FontWeight.SemiBold); Text("Enter the one-time pairing payload shown by SunCode Desktop.", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant); Button(onClick, Modifier.fillMaxWidth()) { Icon(Icons.Default.QrCodeScanner, null); Spacer(Modifier.width(8.dp)); Text("Enter pairing payload") } } } }
@Composable private fun SettingsGroup(title: String, content: @Composable () -> Unit) { Column(Modifier.fillMaxWidth().padding(bottom = 14.dp)) { Text(title.uppercase(), fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant, modifier = Modifier.padding(4.dp, 8.dp)); Card(border = BorderStroke(1.dp, MaterialTheme.colorScheme.outline)) { content() } } }
@Composable private fun SettingRow(icon: ImageVector, label: String, value: String, onClick: () -> Unit) { Row(Modifier.fillMaxWidth().clickable(onClick = onClick).padding(14.dp), verticalAlignment = Alignment.CenterVertically) { Icon(icon, null, Modifier.size(19.dp)); Text(label, Modifier.weight(1f).padding(start = 10.dp), fontSize = 13.sp); Text(value, fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) } }
@Composable private fun ConnectionBanner(text: String, state: HostConnectionState) { Row(Modifier.padding(vertical = 8.dp), verticalAlignment = Alignment.CenterVertically) { StatusDot(state); Text(text, Modifier.padding(start = 7.dp), fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant) } }
@Composable private fun StatusDot(state: Any) { Box(Modifier.size(8.dp).background(if (stateColor(state) == Color.Unspecified) MaterialTheme.colorScheme.onSurfaceVariant else stateColor(state), CircleShape)) }
@Composable private fun DetailLine(label: String, value: String) { Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) { Text(label, fontSize = 11.sp, color = MaterialTheme.colorScheme.onSurfaceVariant); Text(value, fontSize = 11.sp) } }

@Composable
private fun PairingDialog(onConfirm: (String) -> Unit, onScan: (((String) -> Unit) -> Unit)?, errorText: String?, confirming: Boolean, onDismiss: () -> Unit) {
    var payload by remember { mutableStateOf("") }
    var reviewPayload by remember { mutableStateOf(false) }
    AlertDialog(
        onDismissRequest = onDismiss,
        icon = { Icon(if (reviewPayload) Icons.Default.Check else Icons.Default.QrCodeScanner, null) },
        title = { Text(if (reviewPayload) "Confirm pairing" else "Enter pairing payload") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(10.dp)) {
                Text(if (reviewPayload) "Exchange this one-time payload with Remote Server?" else "Scan the QR code or paste the payload from SunCode Desktop.", fontSize = 12.sp)
                if (!reviewPayload && onScan != null) {
                    OutlinedButton(onClick = { onScan { scanned -> payload = scanned; reviewPayload = true } }, modifier = Modifier.fillMaxWidth()) {
                        Icon(Icons.Default.QrCodeScanner, null)
                        Spacer(Modifier.width(8.dp))
                        Text("Open camera")
                    }
                }
                OutlinedTextField(payload, { payload = it }, label = { Text("Pairing payload") }, minLines = 3, modifier = Modifier.fillMaxWidth())
                errorText?.let { Text(it, color = MaterialTheme.colorScheme.error, fontSize = 12.sp) }
            }
        },
        confirmButton = { Button(onClick = { if (reviewPayload) onConfirm(payload) else reviewPayload = true }, enabled = payload.isNotBlank() && !confirming) { Text(if (confirming) "Pairing…" else if (reviewPayload) "Confirm pairing" else "Continue") } },
        dismissButton = { TextButton(onClick = onDismiss, enabled = !confirming) { Text("Cancel") } },
    )
}

@Composable
private fun NewSessionDialog(hosts: List<Host>, onCreate: (String, String) -> Unit, onDismiss: () -> Unit) {
    var title by remember { mutableStateOf("") }
    var message by remember { mutableStateOf("") }
    val host = hosts.firstOrNull()
    val project = host?.projects?.firstOrNull()
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("New session") },
        text = {
            Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text("${host?.name ?: "No Host"} · ${project?.name ?: "No Project"}", fontSize = 12.sp, color = MaterialTheme.colorScheme.onSurfaceVariant)
                OutlinedTextField(title, { title = it }, label = { Text("Session title") }, singleLine = true)
                OutlinedTextField(message, { message = it }, label = { Text("First message") }, minLines = 3)
            }
        },
        confirmButton = { Button(onClick = { onCreate(title, message) }, enabled = host != null && project != null) { Text("Create session") } },
        dismissButton = { TextButton(onClick = onDismiss) { Text("Cancel") } },
    )
}

@Composable
private fun ThemeDialog(theme: ThemePreference, onSelect: (ThemePreference) -> Unit, onDismiss: () -> Unit) {
    AlertDialog(
        onDismissRequest = onDismiss,
        title = { Text("Theme") },
        text = { Column { ThemePreference.entries.forEach { preference -> Row(Modifier.fillMaxWidth().clickable { onSelect(preference) }.padding(vertical = 9.dp), verticalAlignment = Alignment.CenterVertically) { RadioButton(theme == preference, { onSelect(preference) }); Text(preference.displayName()) } } } },
        confirmButton = {},
    )
}

private fun ThemePreference.displayName() = name.lowercase().replaceFirstChar { it.uppercase() }
private fun SessionState.displayName() = when (this) { SessionState.IDLE -> "Idle"; SessionState.RUNNING -> "Running"; SessionState.WAITING_FOR_APPROVAL -> "Waiting for approval"; SessionState.WAITING_FOR_ANSWER -> "Waiting for answer"; SessionState.FAILED -> "Failed" }
private fun HostConnectionState.displayName() = when (this) { HostConnectionState.CONNECTED -> "Connected"; HostConnectionState.CONNECTING -> "Connecting"; HostConnectionState.DEGRADED -> "Degraded"; HostConnectionState.OFFLINE -> "Offline"; HostConnectionState.UNAUTHORIZED -> "Unauthorized"; HostConnectionState.INCOMPATIBLE -> "Incompatible" }
@Composable private fun sessionColor(state: SessionState) = when (state) { SessionState.WAITING_FOR_APPROVAL, SessionState.WAITING_FOR_ANSWER -> Color(0xFF966A25); SessionState.RUNNING -> Color(0xFF4F6D82); SessionState.FAILED -> Color(0xFFB6463F); else -> MaterialTheme.colorScheme.onSurfaceVariant }
@Composable private fun hostColor(state: HostConnectionState) = when (state) { HostConnectionState.CONNECTED -> Color(0xFF4F6D82); HostConnectionState.CONNECTING, HostConnectionState.DEGRADED -> Color(0xFF966A25); HostConnectionState.UNAUTHORIZED, HostConnectionState.INCOMPATIBLE -> Color(0xFFB6463F); HostConnectionState.OFFLINE -> MaterialTheme.colorScheme.onSurfaceVariant }
@Composable private fun stateColor(state: Any) = when (state) { is SessionState -> sessionColor(state); is HostConnectionState -> hostColor(state); else -> Color.Unspecified }
