package ai.suncode.mobile.remote

import ai.suncode.mobile.remote.protocol.ApiBaseRet
import ai.suncode.mobile.remote.protocol.ApprovalResolutionRequest
import ai.suncode.mobile.remote.protocol.CommandAcceptedData
import ai.suncode.mobile.remote.protocol.CreateSessionRequest
import ai.suncode.mobile.remote.protocol.HealthData
import ai.suncode.mobile.remote.protocol.HostDto
import ai.suncode.mobile.remote.protocol.PairingExchangeData
import ai.suncode.mobile.remote.protocol.PairingExchangeRequest
import ai.suncode.mobile.remote.protocol.ProjectsData
import ai.suncode.mobile.remote.protocol.QuestionReplyRequest
import ai.suncode.mobile.remote.protocol.RefreshTokenRequest
import ai.suncode.mobile.remote.protocol.SendMessageRequest
import ai.suncode.mobile.remote.protocol.SessionDetailDto
import ai.suncode.mobile.remote.protocol.SessionPageData
import ai.suncode.mobile.remote.protocol.SessionStreamEvent
import ai.suncode.mobile.remote.protocol.SyncData
import ai.suncode.mobile.remote.protocol.TokenData
import ai.suncode.mobile.remote.protocol.EncryptedPayload
import io.ktor.client.HttpClient
import io.ktor.client.engine.HttpClientEngineFactory
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.plugins.sse.SSE
import io.ktor.client.plugins.sse.SSEClientException
import io.ktor.client.plugins.sse.sse
import io.ktor.client.request.accept
import io.ktor.client.request.bearerAuth
import io.ktor.client.request.get
import io.ktor.client.request.header
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.client.statement.HttpResponse
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.HttpStatusCode
import io.ktor.http.contentType
import io.ktor.http.isSuccess
import io.ktor.serialization.kotlinx.json.json
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.Serializable
import kotlin.random.Random

@Serializable
data class RemoteHostCredentials(
    val endpoint: String,
    val tokens: TokenData? = null,
    val aesKey: String? = null,
)

@Serializable
data class RemoteHostCredentialStore(
    val hosts: Map<String, RemoteHostCredentials> = emptyMap(),
)

interface RemoteTokenStore {
    suspend fun load(hostId: String): RemoteHostCredentials?
    suspend fun save(hostId: String, credentials: RemoteHostCredentials)
    suspend fun clear(hostId: String)
}

/** In-memory store for wiring and tests; production storage must be platform-secure. */
class InMemoryRemoteTokenStore : RemoteTokenStore {
    private val credentials = mutableMapOf<String, RemoteHostCredentials>()

    override suspend fun load(hostId: String): RemoteHostCredentials? = credentials[hostId]

    override suspend fun save(hostId: String, credentials: RemoteHostCredentials) {
        this.credentials[hostId] = credentials
    }

    override suspend fun clear(hostId: String) {
        credentials.remove(hostId)
    }
}

class RemoteHttpException(
    val status: HttpStatusCode,
    val responseCode: Int?,
    override val message: String,
) : IllegalStateException(message)

class KtorRemoteControlClient(
    private val tokenStore: RemoteTokenStore,
    engineFactory: HttpClientEngineFactory<*> = platformHttpClientEngine(),
    private val json: Json = Json {
        ignoreUnknownKeys = true
        explicitNulls = false
    },
) : RemoteControlClient {
    private val http = HttpClient(engineFactory) {
        install(ContentNegotiation) {
            json(json)
        }
        install(SSE) {
            maxReconnectionAttempts = 0
        }
    }
    private val refreshMutex = Mutex()

    override suspend fun configureHost(hostId: String, endpoint: String, aesKey: String?) {
        val normalizedEndpoint = endpoint.trimEnd('/')
        require(normalizedEndpoint.startsWith("http://") || normalizedEndpoint.startsWith("https://")) {
            "Remote endpoint must use http or https"
        }
        val existing = tokenStore.load(hostId)
        tokenStore.save(
            hostId,
            RemoteHostCredentials(
                endpoint = normalizedEndpoint,
                tokens = existing?.tokens,
                aesKey = aesKey ?: existing?.aesKey,
            ),
        )
    }

    /** Release the engine when the application scope is shut down. */
    fun close() {
        http.close()
    }

    override suspend fun health(hostId: String): ApiBaseRet<HealthData> {
        val response = http.get("${endpoint(hostId)}/v1/mobile/health") { routingHeaders(hostId) }
        val body = decryptPayload(hostId, response.bodyAsText())
        val decoded = body.takeIf(String::isNotBlank)?.let { json.decodeFromString<ApiBaseRet<HealthData>>(it) }
        if (!response.status.isSuccess() || decoded?.code?.let { it != 0 } == true) {
            throw RemoteHttpException(response.status, decoded?.code, decoded?.message ?: "Remote Server request failed")
        }
        return decoded ?: ApiBaseRet()
    }

    override suspend fun exchangePairing(hostId: String, request: PairingExchangeRequest): ApiBaseRet<PairingExchangeData> {
        val response = envelope<PairingExchangeData> {
            postJson("${endpoint(hostId)}/v1/mobile/pairings/exchange", request, hostId = hostId)
        }
        response.data?.let { pairing ->
            val existing = tokenStore.load(hostId)
            tokenStore.save(
                hostId,
                RemoteHostCredentials(
                    endpoint = existing?.endpoint ?: error("Remote endpoint is not configured"),
                    tokens = TokenData(pairing.accessToken, pairing.refreshToken, pairing.accessTokenExpiresAt),
                    aesKey = existing?.aesKey,
                ),
            )
        }
        return response
    }

    override suspend fun refreshToken(hostId: String, request: RefreshTokenRequest): ApiBaseRet<TokenData> {
        val response = envelope<TokenData> { postJson("${endpoint(hostId)}/v1/mobile/auth/refresh", request, hostId = hostId) }
        response.data?.let { tokenStore.updateTokens(hostId, it) }
        return response
    }

    override suspend fun logout(hostId: String) {
        try {
            authenticatedEmptyResponse(hostId) { token -> post("${endpoint(hostId)}/v1/mobile/auth/logout") { authenticated(token, hostId) } }
        } finally {
            tokenStore.clear(hostId)
        }
    }

    override suspend fun getHost(hostId: String): ApiBaseRet<HostDto> {
        return authenticatedEnvelope(hostId) { token -> get("${endpoint(hostId)}/v1/mobile/hosts/${hostId.pathSegment()}") { authenticated(token, hostId) } }
    }

    override suspend fun listProjects(hostId: String): ApiBaseRet<ProjectsData> {
        return authenticatedEnvelope(hostId) { token -> get("${endpoint(hostId)}/v1/mobile/hosts/${hostId.pathSegment()}/projects") { authenticated(token, hostId) } }
    }

    override suspend fun listSessions(hostId: String, projectId: String?, cursor: String?, limit: Int?): ApiBaseRet<SessionPageData> {
        return authenticatedEnvelope(hostId) { token ->
            get("${endpoint(hostId)}/v1/mobile/hosts/${hostId.pathSegment()}/sessions") {
                authenticated(token, hostId)
                url {
                    projectId?.let { parameters.append("projectId", it) }
                    cursor?.let { parameters.append("cursor", it) }
                    limit?.let { parameters.append("limit", it.toString()) }
                }
            }
        }
    }

    override suspend fun createSession(hostId: String, request: CreateSessionRequest): ApiBaseRet<CommandAcceptedData> {
        return authenticatedEnvelope(hostId) { token -> postJson("${endpoint(hostId)}/v1/mobile/hosts/${hostId.pathSegment()}/sessions", request, hostId = hostId, token = token, encrypted = true, query = mapOf("projectId" to request.projectId)) }
    }

    override suspend fun getSession(hostId: String, sessionId: String): ApiBaseRet<SessionDetailDto> {
        return authenticatedEnvelope(hostId) { token -> get("${endpoint(hostId)}/v1/mobile/hosts/${hostId.pathSegment()}/sessions/${sessionId.pathSegment()}") { authenticated(token, hostId) } }
    }

    override suspend fun sendMessage(hostId: String, sessionId: String, request: SendMessageRequest): ApiBaseRet<CommandAcceptedData> {
        return authenticatedEnvelope(hostId) { token -> postJson("${endpoint(hostId)}/v1/mobile/hosts/${hostId.pathSegment()}/sessions/${sessionId.pathSegment()}/messages", request, hostId = hostId, token = token, encrypted = true) }
    }

    override suspend fun resolveApproval(hostId: String, sessionId: String, approvalId: String, request: ApprovalResolutionRequest): ApiBaseRet<CommandAcceptedData> {
        return authenticatedEnvelope(hostId) { token -> postJson("${endpoint(hostId)}/v1/mobile/hosts/${hostId.pathSegment()}/sessions/${sessionId.pathSegment()}/approvals/${approvalId.pathSegment()}", request, hostId = hostId, token = token, encrypted = true) }
    }

    override suspend fun replyQuestion(hostId: String, sessionId: String, questionId: String, request: QuestionReplyRequest): ApiBaseRet<CommandAcceptedData> {
        return authenticatedEnvelope(hostId) { token -> postJson("${endpoint(hostId)}/v1/mobile/hosts/${hostId.pathSegment()}/sessions/${sessionId.pathSegment()}/questions/${questionId.pathSegment()}/reply", request, hostId = hostId, token = token, encrypted = true) }
    }

    override suspend fun cancelTurn(hostId: String, sessionId: String): ApiBaseRet<CommandAcceptedData> {
        return authenticatedEnvelope(hostId) { token -> post("${endpoint(hostId)}/v1/mobile/hosts/${hostId.pathSegment()}/sessions/${sessionId.pathSegment()}/cancel") { authenticated(token, hostId) } }
    }

    override suspend fun retryLastTurn(hostId: String, sessionId: String): ApiBaseRet<CommandAcceptedData> {
        return authenticatedEnvelope(hostId) { token -> post("${endpoint(hostId)}/v1/mobile/hosts/${hostId.pathSegment()}/sessions/${sessionId.pathSegment()}/retry") { authenticated(token, hostId) } }
    }

    override suspend fun sync(hostId: String, cursor: String?, limit: Int?): ApiBaseRet<SyncData> {
        return authenticatedEnvelope(hostId) { token ->
            get("${endpoint(hostId)}/v1/mobile/hosts/${hostId.pathSegment()}/sync") {
                authenticated(token, hostId)
                url {
                    cursor?.let { parameters.append("cursor", it) }
                    limit?.let { parameters.append("limit", it.toString()) }
                }
            }
        }
    }

    override fun observeSessionEvents(hostId: String, sessionId: String, lastEventId: String?): Flow<SessionStreamEvent> = flow {
        var token = accessTokenOrRefresh(hostId)
        val url = "${endpoint(hostId)}/v1/mobile/hosts/${hostId.pathSegment()}/sessions/${sessionId.pathSegment()}/events"
        try {
            openEventStream(url, hostId, token, lastEventId) { emit(it) }
        } catch (failure: SSEClientException) {
            val status = failure.response?.status ?: HttpStatusCode.InternalServerError
            if (status != HttpStatusCode.Unauthorized) {
                throw RemoteHttpException(status, null, "Remote Session event stream failed")
            }
            token = refreshAccessToken(hostId, token)
            openEventStream(url, hostId, token, lastEventId) { emit(it) }
        }
    }

    private suspend fun openEventStream(
        url: String,
        hostId: String,
        token: String,
        lastEventId: String?,
        onEvent: suspend (SessionStreamEvent) -> Unit,
    ) {
        http.sse(url, request = {
            authenticated(token, hostId)
            lastEventId?.let { header("Last-Event-ID", it) }
        }) {
            incoming.collect { event ->
                event.data?.let { data ->
                    onEvent(SessionStreamEvent(event.id, event.event, decryptPayload(hostId, data)))
                }
            }
        }
    }

    private suspend inline fun <reified T> envelope(crossinline request: suspend HttpClient.() -> HttpResponse): ApiBaseRet<T> {
        val response = request(http)
        val body = response.bodyAsText()
        val decoded = body.takeIf(String::isNotBlank)?.let { json.decodeFromString<ApiBaseRet<T>>(it) }
        if (!response.status.isSuccess() || decoded?.code?.let { it != 0 } == true) {
            throw RemoteHttpException(response.status, decoded?.code, decoded?.message ?: "Remote Server request failed")
        }
        return decoded ?: ApiBaseRet()
    }

    private suspend inline fun <reified T> authenticatedEnvelope(hostId: String, crossinline request: suspend HttpClient.(String) -> HttpResponse): ApiBaseRet<T> {
        var token = accessTokenOrRefresh(hostId)
        var response = request(http, token)
        if (response.status == HttpStatusCode.Unauthorized) {
            response.bodyAsText()
            token = refreshAccessToken(hostId, token)
            response = request(http, token)
        }
        return decodeEnvelope(hostId, response)
    }

    private suspend inline fun authenticatedEmptyResponse(hostId: String, crossinline request: suspend HttpClient.(String) -> HttpResponse) {
        var token = accessTokenOrRefresh(hostId)
        var response = request(http, token)
        if (response.status == HttpStatusCode.Unauthorized) {
            response.bodyAsText()
            token = refreshAccessToken(hostId, token)
            response = request(http, token)
        }
        if (!response.status.isSuccess()) {
            val body = response.bodyAsText()
            val decoded = body.takeIf(String::isNotBlank)?.let { json.decodeFromString<ApiBaseRet<JsonObject>>(it) }
            throw RemoteHttpException(response.status, decoded?.code, decoded?.message ?: "Remote Server request failed")
        }
    }

    private suspend inline fun <reified T> decodeEnvelope(hostId: String, response: HttpResponse): ApiBaseRet<T> {
        val body = response.bodyAsText()
        val decodedBody = decryptPayload(hostId, body)
        val decoded = decodedBody.takeIf(String::isNotBlank)?.let { json.decodeFromString<ApiBaseRet<T>>(it) }
        if (!response.status.isSuccess() || decoded?.code?.let { it != 0 } == true) {
            throw RemoteHttpException(response.status, decoded?.code, decoded?.message ?: "Remote Server request failed")
        }
        return decoded ?: ApiBaseRet()
    }

    private fun io.ktor.client.request.HttpRequestBuilder.authenticated(token: String, hostId: String) {
        bearerAuth(token)
        header("X-Host-Id", hostId)
        header("X-Request-Id", requestId())
    }

    private suspend inline fun <reified T> HttpClient.postJson(url: String, body: T, hostId: String, token: String? = null, encrypted: Boolean = false, query: Map<String, String> = emptyMap()): HttpResponse =
        post(url) {
            contentType(ContentType.Application.Json)
            accept(ContentType.Application.Json)
            token?.let { bearerAuth(it) }
            header("X-Host-Id", hostId)
            header("X-Request-Id", requestId())
            url { query.forEach { (key, value) -> parameters.append(key, value) } }
            setBody(if (encrypted) encryptedBody(hostId, body) else body)
        }

    private suspend inline fun <reified T> encryptedBody(hostId: String, body: T): EncryptedPayload {
        val key = tokenStore.load(hostId)?.aesKey ?: error("Remote E2E key is unavailable")
        return EncryptedPayload(E2eCrypto.encrypt(key, json.encodeToString(body).encodeToByteArray()))
    }

    private suspend fun decryptPayload(hostId: String, body: String): String {
        val wrapper = runCatching { json.decodeFromString<EncryptedPayload>(body) }.getOrNull() ?: return body
        val key = tokenStore.load(hostId)?.aesKey ?: error("Remote E2E key is unavailable")
        return E2eCrypto.decrypt(key, wrapper.encPayload).decodeToString()
    }

    private suspend fun accessTokenOrRefresh(hostId: String): String =
        tokenStore.load(hostId)?.tokens?.accessToken ?: refreshAccessToken(hostId, null)

    private suspend fun refreshAccessToken(hostId: String, rejectedToken: String?): String = refreshMutex.withLock {
        tokenStore.load(hostId)?.tokens?.accessToken?.takeIf { it != rejectedToken }?.let { return@withLock it }
        val refreshToken = tokenStore.load(hostId)?.tokens?.refreshToken
            ?: throw IllegalStateException("An access token or refresh token is required")
        try {
            val response = envelope<TokenData> {
                postJson("${endpoint(hostId)}/v1/mobile/auth/refresh", RefreshTokenRequest(refreshToken), hostId = hostId)
            }
            val tokens = response.data ?: throw IllegalStateException("Remote Server returned no access token")
            tokenStore.updateTokens(hostId, tokens)
            tokens.accessToken
        } catch (failure: Throwable) {
            if (failure is RemoteHttpException && failure.status == HttpStatusCode.Unauthorized) tokenStore.clear(hostId)
            throw failure
        }
    }

    private fun String.pathSegment(): String = encodeURLPathPart()

    private fun requestId(): String = "mobile-" + Random.nextLong().toString(16)
    private fun io.ktor.client.request.HttpRequestBuilder.routingHeaders(hostId: String) {
        header("X-Host-Id", hostId)
        header("X-Request-Id", requestId())
    }

    private suspend fun endpoint(hostId: String): String =
        tokenStore.load(hostId)?.endpoint ?: throw IllegalStateException("Remote endpoint is not configured for host $hostId")
}

private suspend fun RemoteTokenStore.updateTokens(hostId: String, tokens: TokenData) {
    val existing = load(hostId) ?: error("Remote endpoint is not configured")
    save(hostId, existing.copy(tokens = tokens))
}

private fun String.encodeURLPathPart(): String = buildString {
    for (byte in encodeToByteArray()) {
        val c = byte.toInt().and(0xff).toChar()
        if (c.isLetterOrDigit() || c in "-._~") append(c) else append('%').append(byte.toInt().and(0xff).toString(16).uppercase().padStart(2, '0'))
    }
}
