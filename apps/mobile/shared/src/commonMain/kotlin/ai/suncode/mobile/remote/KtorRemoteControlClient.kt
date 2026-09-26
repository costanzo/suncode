package ai.suncode.mobile.remote

import ai.suncode.mobile.remote.protocol.AgentEventEnvelope
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
import ai.suncode.mobile.remote.protocol.SyncData
import ai.suncode.mobile.remote.protocol.TokenData
import io.ktor.client.HttpClient
import io.ktor.client.engine.HttpClientEngineFactory
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.client.plugins.websocket.webSocket
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
import io.ktor.websocket.Frame
import io.ktor.websocket.readText
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.flow
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.JsonObject

interface RemoteTokenStore {
    suspend fun accessToken(): String?
    suspend fun refreshToken(): String?
    suspend fun save(tokens: TokenData)
    suspend fun clear()
}

/** In-memory store for wiring and tests; production storage must be platform-secure. */
class InMemoryRemoteTokenStore : RemoteTokenStore {
    private var tokens: TokenData? = null

    override suspend fun accessToken(): String? = tokens?.accessToken

    override suspend fun refreshToken(): String? = tokens?.refreshToken

    override suspend fun save(tokens: TokenData) {
        this.tokens = tokens
    }

    override suspend fun clear() {
        tokens = null
    }
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
        install(io.ktor.client.plugins.websocket.WebSockets)
    }
    private val refreshMutex = Mutex()

    /** Release the engine when the application scope is shut down. */
    fun close() {
        http.close()
    }

    override suspend fun health(): ApiBaseRet<HealthData> = envelope { get("$baseUrl/v1/health") }

    override suspend fun exchangePairing(request: PairingExchangeRequest): ApiBaseRet<PairingExchangeData> {
        val response = envelope<PairingExchangeData> { postJson("$baseUrl/v1/pairings/exchange", request) }
        response.data?.let { pairing ->
            tokenStore.save(TokenData(pairing.accessToken, pairing.refreshToken, pairing.accessTokenExpiresAt))
        }
        return response
    }

    override suspend fun refreshToken(request: RefreshTokenRequest): ApiBaseRet<TokenData> {
        val response = envelope<TokenData> { postJson("$baseUrl/v1/auth/refresh", request) }
        response.data?.let { tokenStore.save(it) }
        return response
    }

    override suspend fun logout(idempotencyKey: String) {
        try {
            authenticatedEmptyResponse { token -> post("$baseUrl/v1/auth/logout") { authenticated(token, idempotencyKey) } }
        } finally {
            tokenStore.clear()
        }
    }

    override suspend fun getHost(hostId: String): ApiBaseRet<HostDto> {
        return authenticatedEnvelope { token -> get("$baseUrl/v1/hosts/${hostId.pathSegment()}") { authenticated(token) } }
    }

    override suspend fun listProjects(hostId: String): ApiBaseRet<ProjectsData> {
        return authenticatedEnvelope { token -> get("$baseUrl/v1/hosts/${hostId.pathSegment()}/projects") { authenticated(token) } }
    }

    override suspend fun listSessions(hostId: String?, projectId: String?, cursor: String?, limit: Int?): ApiBaseRet<SessionPageData> {
        return authenticatedEnvelope { token ->
            get("$baseUrl/v1/sessions") {
                authenticated(token)
                url {
                    hostId?.let { parameters.append("hostId", it) }
                    projectId?.let { parameters.append("projectId", it) }
                    cursor?.let { parameters.append("cursor", it) }
                    limit?.let { parameters.append("limit", it.toString()) }
                }
            }
        }
    }

    override suspend fun createSession(request: CreateSessionRequest, idempotencyKey: String) {
        authenticatedEmptyResponse { token -> postJson("$baseUrl/v1/sessions", request, idempotencyKey, token) }
    }

    override suspend fun getSession(sessionId: String): ApiBaseRet<SessionDetailDto> {
        return authenticatedEnvelope { token -> get("$baseUrl/v1/sessions/${sessionId.pathSegment()}") { authenticated(token) } }
    }

    override suspend fun sendMessage(sessionId: String, request: SendMessageRequest, idempotencyKey: String): ApiBaseRet<CommandAcceptedData> {
        return authenticatedEnvelope { token -> postJson("$baseUrl/v1/sessions/${sessionId.pathSegment()}/messages", request, idempotencyKey, token) }
    }

    override suspend fun resolveApproval(sessionId: String, approvalId: String, request: ApprovalResolutionRequest, idempotencyKey: String): ApiBaseRet<CommandAcceptedData> {
        return authenticatedEnvelope { token -> postJson("$baseUrl/v1/sessions/${sessionId.pathSegment()}/approvals/${approvalId.pathSegment()}", request, idempotencyKey, token) }
    }

    override suspend fun replyQuestion(sessionId: String, questionId: String, request: QuestionReplyRequest, idempotencyKey: String): ApiBaseRet<CommandAcceptedData> {
        return authenticatedEnvelope { token -> postJson("$baseUrl/v1/sessions/${sessionId.pathSegment()}/questions/${questionId.pathSegment()}/reply", request, idempotencyKey, token) }
    }

    override suspend fun cancelTurn(sessionId: String, idempotencyKey: String): ApiBaseRet<CommandAcceptedData> {
        return authenticatedEnvelope { token -> post("$baseUrl/v1/sessions/${sessionId.pathSegment()}/cancel") { authenticated(token, idempotencyKey) } }
    }

    override suspend fun retryLastTurn(sessionId: String, idempotencyKey: String): ApiBaseRet<CommandAcceptedData> {
        return authenticatedEnvelope { token -> post("$baseUrl/v1/sessions/${sessionId.pathSegment()}/retry") { authenticated(token, idempotencyKey) } }
    }

    override suspend fun sync(cursor: String?, limit: Int?): ApiBaseRet<SyncData> {
        return authenticatedEnvelope { token ->
            get("$baseUrl/v1/sync") {
                authenticated(token)
                url {
                    cursor?.let { parameters.append("cursor", it) }
                    limit?.let { parameters.append("limit", it.toString()) }
                }
            }
        }
    }

    override fun observeEvents(): Flow<AgentEventEnvelope> = flow {
        val token = accessTokenOrRefresh()
        val websocketUrl = baseUrl.replaceFirst("https://", "wss://").replaceFirst("http://", "ws://") + "/v1/ws"
        http.webSocket(websocketUrl, request = { bearerAuth(token) }) {
            for (frame in incoming) {
                if (frame is Frame.Text) {
                    emit(json.decodeFromString<AgentEventEnvelope>(frame.readText()))
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

    private fun io.ktor.client.request.HttpRequestBuilder.authenticated(token: String, idempotencyKey: String? = null) {
        bearerAuth(token)
        idempotencyKey?.let { header("Idempotency-Key", it) }
    }

    private suspend inline fun <reified T> HttpClient.postJson(url: String, body: T, idempotencyKey: String? = null, token: String? = null): HttpResponse =
        post(url) {
            contentType(ContentType.Application.Json)
            accept(ContentType.Application.Json)
            token?.let { bearerAuth(it) }
            idempotencyKey?.let { header("Idempotency-Key", it) }
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
                postJson("$baseUrl/v1/auth/refresh", RefreshTokenRequest(refreshToken))
            }
            response.data?.accessToken ?: throw IllegalStateException("Remote Server returned no access token")
        } catch (failure: Throwable) {
            if (failure is RemoteHttpException && failure.status == HttpStatusCode.Unauthorized) tokenStore.clear()
            throw failure
        }
    }

    private fun String.pathSegment(): String = encodeURLPathPart()
}

private fun String.encodeURLPathPart(): String = buildString {
    for (byte in encodeToByteArray()) {
        val c = byte.toInt().and(0xff).toChar()
        if (c.isLetterOrDigit() || c in "-._~") append(c) else append('%').append(byte.toInt().and(0xff).toString(16).uppercase().padStart(2, '0'))
    }
}
