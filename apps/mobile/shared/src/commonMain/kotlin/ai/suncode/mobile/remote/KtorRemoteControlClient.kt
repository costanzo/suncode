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
import kotlin.random.Random

interface RemoteTokenStore {
    suspend fun accessToken(): String?
    suspend fun refreshToken(): String?
    suspend fun save(tokens: TokenData)
    suspend fun clear()
    /** Host binding and E2E key are persisted alongside credentials by production stores. */
    suspend fun hostId(): String? = null
    suspend fun saveHostId(hostId: String) = Unit
    suspend fun saveAesKey(aesKey: String) = Unit
}

/** In-memory store for wiring and tests; production storage must be platform-secure. */
class InMemoryRemoteTokenStore : RemoteTokenStore {
    private var tokens: TokenData? = null
    private var pairedHostId: String? = null
    private var aesKey: String? = null

    override suspend fun accessToken(): String? = tokens?.accessToken

    override suspend fun refreshToken(): String? = tokens?.refreshToken

    override suspend fun save(tokens: TokenData) {
        this.tokens = tokens
    }

    override suspend fun clear() {
        tokens = null
        pairedHostId = null
        aesKey = null
    }

    override suspend fun hostId(): String? = pairedHostId
    override suspend fun saveHostId(hostId: String) { pairedHostId = hostId }
    override suspend fun saveAesKey(aesKey: String) { this.aesKey = aesKey }
}

class RemoteHttpException(
    val status: HttpStatusCode,
    val responseCode: Int?,
    override val message: String,
) : IllegalStateException(message)

class KtorRemoteControlClient(
    baseUrl: String,
    private val tokenStore: RemoteTokenStore,
    engineFactory: HttpClientEngineFactory<*> = platformHttpClientEngine(),
    private val json: Json = Json {
        ignoreUnknownKeys = true
        explicitNulls = false
    },
) : RemoteControlClient {
    private val baseUrl = baseUrl.trimEnd('/')
    private val http = HttpClient(engineFactory) {
        install(ContentNegotiation) {
            json(json)
        }
        install(SSE) {
            maxReconnectionAttempts = 0
        }
    }
    private val refreshMutex = Mutex()

    override suspend fun rememberPairingKey(key: String) = tokenStore.saveAesKey(key)

    /** Release the engine when the application scope is shut down. */
    fun close() {
        http.close()
    }

    override suspend fun health(): ApiBaseRet<HealthData> = envelope { get("$baseUrl/v1/mobile/health") { routingHeaders(activeHostIdOrUnknown()) } }

    override suspend fun exchangePairing(hostId: String, request: PairingExchangeRequest): ApiBaseRet<PairingExchangeData> {
        currentHostId = hostId
        val response = envelope<PairingExchangeData> { postJson("$baseUrl/v1/mobile/pairings/exchange", request, hostId = hostId) }
        response.data?.let { pairing ->
            tokenStore.save(TokenData(pairing.accessToken, pairing.refreshToken, pairing.accessTokenExpiresAt))
            tokenStore.saveHostId(hostId)
        }
        return response
    }

    override suspend fun refreshToken(request: RefreshTokenRequest): ApiBaseRet<TokenData> {
        val hostId = activeHostId()
        val response = envelope<TokenData> { postJson("$baseUrl/v1/mobile/auth/refresh", request, hostId = hostId) }
        response.data?.let { tokenStore.save(it) }
        return response
    }

    override suspend fun logout() {
        try {
            val hostId = activeHostId()
            authenticatedEmptyResponse { token -> post("$baseUrl/v1/mobile/auth/logout") { authenticated(token, hostId) } }
        } finally {
            tokenStore.clear()
        }
    }

    override suspend fun getHost(hostId: String): ApiBaseRet<HostDto> {
        return authenticatedEnvelope { token -> get("$baseUrl/v1/mobile/hosts/${hostId.pathSegment()}") { authenticated(token, hostId) } }
    }

    override suspend fun listProjects(hostId: String): ApiBaseRet<ProjectsData> {
        return authenticatedEnvelope { token -> get("$baseUrl/v1/mobile/hosts/${hostId.pathSegment()}/projects") { authenticated(token, hostId) } }
    }

    override suspend fun listSessions(hostId: String, projectId: String?, cursor: String?, limit: Int?): ApiBaseRet<SessionPageData> {
        return authenticatedEnvelope { token ->
            get("$baseUrl/v1/mobile/hosts/${hostId.pathSegment()}/sessions") {
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
        return authenticatedEnvelope { token -> postJson("$baseUrl/v1/mobile/hosts/${hostId.pathSegment()}/sessions", request, hostId = hostId, token = token) }
    }

    override suspend fun getSession(hostId: String, sessionId: String): ApiBaseRet<SessionDetailDto> {
        return authenticatedEnvelope { token -> get("$baseUrl/v1/mobile/hosts/${hostId.pathSegment()}/sessions/${sessionId.pathSegment()}") { authenticated(token, hostId) } }
    }

    override suspend fun sendMessage(hostId: String, sessionId: String, request: SendMessageRequest): ApiBaseRet<CommandAcceptedData> {
        return authenticatedEnvelope { token -> postJson("$baseUrl/v1/mobile/hosts/${hostId.pathSegment()}/sessions/${sessionId.pathSegment()}/messages", request, hostId = hostId, token = token) }
    }

    override suspend fun resolveApproval(hostId: String, sessionId: String, approvalId: String, request: ApprovalResolutionRequest): ApiBaseRet<CommandAcceptedData> {
        return authenticatedEnvelope { token -> postJson("$baseUrl/v1/mobile/hosts/${hostId.pathSegment()}/sessions/${sessionId.pathSegment()}/approvals/${approvalId.pathSegment()}", request, hostId = hostId, token = token) }
    }

    override suspend fun replyQuestion(hostId: String, sessionId: String, questionId: String, request: QuestionReplyRequest): ApiBaseRet<CommandAcceptedData> {
        return authenticatedEnvelope { token -> postJson("$baseUrl/v1/mobile/hosts/${hostId.pathSegment()}/sessions/${sessionId.pathSegment()}/questions/${questionId.pathSegment()}/reply", request, hostId = hostId, token = token) }
    }

    override suspend fun cancelTurn(hostId: String, sessionId: String): ApiBaseRet<CommandAcceptedData> {
        return authenticatedEnvelope { token -> post("$baseUrl/v1/mobile/hosts/${hostId.pathSegment()}/sessions/${sessionId.pathSegment()}/cancel") { authenticated(token, hostId) } }
    }

    override suspend fun retryLastTurn(hostId: String, sessionId: String): ApiBaseRet<CommandAcceptedData> {
        return authenticatedEnvelope { token -> post("$baseUrl/v1/mobile/hosts/${hostId.pathSegment()}/sessions/${sessionId.pathSegment()}/retry") { authenticated(token, hostId) } }
    }

    override suspend fun sync(hostId: String, cursor: String?, limit: Int?): ApiBaseRet<SyncData> {
        return authenticatedEnvelope { token ->
            get("$baseUrl/v1/mobile/hosts/${hostId.pathSegment()}/sync") {
                authenticated(token, hostId)
                url {
                    cursor?.let { parameters.append("cursor", it) }
                    limit?.let { parameters.append("limit", it.toString()) }
                }
            }
        }
    }

    override fun observeSessionEvents(hostId: String, sessionId: String, lastEventId: String?): Flow<SessionStreamEvent> = flow {
        var token = accessTokenOrRefresh()
        val url = "$baseUrl/v1/mobile/hosts/${hostId.pathSegment()}/sessions/${sessionId.pathSegment()}/events"
        try {
            openEventStream(url, hostId, token, lastEventId) { emit(it) }
        } catch (failure: SSEClientException) {
            val status = failure.response?.status ?: HttpStatusCode.InternalServerError
            if (status != HttpStatusCode.Unauthorized) {
                throw RemoteHttpException(status, null, "Remote Session event stream failed")
            }
            token = refreshAccessToken(token)
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
                    onEvent(SessionStreamEvent(event.id, event.event, data))
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

    private suspend inline fun <reified T> authenticatedEnvelope(crossinline request: suspend HttpClient.(String) -> HttpResponse): ApiBaseRet<T> {
        var token = accessTokenOrRefresh()
        var response = request(http, token)
        if (response.status == HttpStatusCode.Unauthorized) {
            response.bodyAsText()
            token = refreshAccessToken(token)
            response = request(http, token)
        }
        return decodeEnvelope(response)
    }

    private suspend inline fun authenticatedEmptyResponse(crossinline request: suspend HttpClient.(String) -> HttpResponse) {
        var token = accessTokenOrRefresh()
        var response = request(http, token)
        if (response.status == HttpStatusCode.Unauthorized) {
            response.bodyAsText()
            token = refreshAccessToken(token)
            response = request(http, token)
        }
        if (!response.status.isSuccess()) {
            val body = response.bodyAsText()
            val decoded = body.takeIf(String::isNotBlank)?.let { json.decodeFromString<ApiBaseRet<JsonObject>>(it) }
            throw RemoteHttpException(response.status, decoded?.code, decoded?.message ?: "Remote Server request failed")
        }
    }

    private suspend inline fun <reified T> decodeEnvelope(response: HttpResponse): ApiBaseRet<T> {
        val body = response.bodyAsText()
        val decoded = body.takeIf(String::isNotBlank)?.let { json.decodeFromString<ApiBaseRet<T>>(it) }
        if (!response.status.isSuccess() || decoded?.code?.let { it != 0 } == true) {
            throw RemoteHttpException(response.status, decoded?.code, decoded?.message ?: "Remote Server request failed")
        }
        return decoded ?: ApiBaseRet()
    }

    private fun io.ktor.client.request.HttpRequestBuilder.authenticated(token: String, hostId: String?) {
        bearerAuth(token)
        header("X-Host-Id", hostId ?: currentHostId.ifBlank { "unknown" })
        header("X-Request-Id", requestId())
    }

    private suspend inline fun <reified T> HttpClient.postJson(url: String, body: T, hostId: String? = null, token: String? = null): HttpResponse =
        post(url) {
            contentType(ContentType.Application.Json)
            accept(ContentType.Application.Json)
            token?.let { bearerAuth(it) }
            header("X-Host-Id", hostId ?: currentHostId)
            header("X-Request-Id", requestId())
            setBody(body)
        }

    private suspend fun accessTokenOrRefresh(): String =
        tokenStore.accessToken() ?: refreshAccessToken(null)

    private suspend fun refreshAccessToken(rejectedToken: String?): String = refreshMutex.withLock {
        tokenStore.accessToken()?.takeIf { it != rejectedToken }?.let { return@withLock it }
        val refreshToken = tokenStore.refreshToken()
            ?: throw IllegalStateException("An access token or refresh token is required")
        try {
            val response = envelope<TokenData> {
                postJson("$baseUrl/v1/mobile/auth/refresh", RefreshTokenRequest(refreshToken), hostId = activeHostId())
            }
            response.data?.accessToken ?: throw IllegalStateException("Remote Server returned no access token")
        } catch (failure: Throwable) {
            if (failure is RemoteHttpException && failure.status == HttpStatusCode.Unauthorized) tokenStore.clear()
            throw failure
        }
    }

    private fun String.pathSegment(): String = encodeURLPathPart()

    private var currentHostId: String = ""
    private suspend fun activeHostId(): String =
        currentHostId.takeIf { it.isNotBlank() } ?: tokenStore.hostId()?.also { currentHostId = it }
        ?: throw IllegalStateException("A paired host is required")
    private suspend fun activeHostIdOrUnknown(): String =
        currentHostId.takeIf { it.isNotBlank() } ?: tokenStore.hostId() ?: "unknown"
    private fun requestId(): String = "mobile-" + Random.nextLong().toString(16)
    private fun io.ktor.client.request.HttpRequestBuilder.routingHeaders(hostId: String?) {
        header("X-Host-Id", hostId ?: currentHostId.ifBlank { "unknown" })
        header("X-Request-Id", requestId())
    }
}

private fun String.encodeURLPathPart(): String = buildString {
    for (byte in encodeToByteArray()) {
        val c = byte.toInt().and(0xff).toChar()
        if (c.isLetterOrDigit() || c in "-._~") append(c) else append('%').append(byte.toInt().and(0xff).toString(16).uppercase().padStart(2, '0'))
    }
}
