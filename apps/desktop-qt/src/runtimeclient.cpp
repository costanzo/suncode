#include "runtimeclient.h"
#include "runtime_sdk.h"
#include "runtimeclient_support.h"

#include <QFutureWatcher>
#include <QGuiApplication>
#include <QJsonDocument>
#include <QDebug>
#include <QClipboard>
#include <QMutex>
#include <QMutexLocker>
#include <QRegularExpression>
#include <QUrlQuery>
#include <QUuid>
#include <QtConcurrent/QtConcurrent>

#include <functional>

namespace {
QMutex sharedRuntimeMutex;
SuncodeRuntimeHandle *sharedHandle = nullptr;
QString sharedHandleError;
QString sharedThemeMode = QStringLiteral("dark");
QSet<RuntimeClient *> sharedThemeClients;
}

RuntimeClient::RuntimeClient(QObject *parent)
    : QObject(parent)
{
    QTimer::singleShot(0, this, &RuntimeClient::connectToRuntime);
}

RuntimeClient::~RuntimeClient()
{
    {
        QMutexLocker locker(&sharedRuntimeMutex);
        sharedThemeClients.remove(this);
    }
    closeEventSubscription();
}

SuncodeRuntimeHandle *RuntimeClient::sharedRuntimeHandle(QString *error)
{
    QMutexLocker locker(&sharedRuntimeMutex);
    if (!sharedHandle && sharedHandleError.isEmpty()) {
        char *sdkError = nullptr;
        sharedHandle = suncode_runtime_sdk_open_default(&sdkError);
        if (!sharedHandle) {
            sharedHandleError = takeSdkString(sdkError);
        }
    }
    if (error) *error = sharedHandleError;
    return sharedHandle;
}

QString RuntimeClient::baseUrl() const
{
    return m_baseUrl;
}

void RuntimeClient::setBaseUrl(const QString &value)
{
    const QString normalized = value.trimmed().remove(QRegularExpression(QStringLiteral("/$")));
    if (normalized == m_baseUrl) {
        return;
    }
    m_baseUrl = normalized;
    emit baseUrlChanged();
}

QString RuntimeClient::projectId() const
{
    return m_projectId;
}

void RuntimeClient::setProjectId(const QString &value)
{
    const QString normalized = value.trimmed();
    if (normalized == m_projectId) {
        return;
    }
    m_projectId = normalized;
    emit projectIdChanged();
}

QString RuntimeClient::sessionId() const
{
    return m_sessionId;
}

QString RuntimeClient::sessionTitle() const
{
    return m_sessionTitle;
}

QString RuntimeClient::activeTurnId() const
{
    return m_activeTurnId;
}

QString RuntimeClient::selectedModel() const
{
    return m_selectedModel;
}

void RuntimeClient::setSelectedModel(const QString &value)
{
    const QString normalized = value.trimmed();
    if (normalized.isEmpty() || normalized == m_selectedModel) {
        return;
    }
    m_selectedModel = normalized;
    emit selectedModelChanged();
}

QString RuntimeClient::themeMode() const
{
    QMutexLocker locker(&sharedRuntimeMutex);
    if (m_themeMode != sharedThemeMode) {
        const_cast<RuntimeClient *>(this)->m_themeMode = sharedThemeMode;
    }
    return m_themeMode;
}

void RuntimeClient::setThemeMode(const QString &value)
{
    const QString normalized = value.trimmed().toLower();
    if (normalized != QStringLiteral("light") && normalized != QStringLiteral("dark")) {
        return;
    }
    QSet<RuntimeClient *> clients;
    {
        QMutexLocker locker(&sharedRuntimeMutex);
        if (sharedThemeMode == normalized) {
            if (m_themeMode != normalized) {
                m_themeMode = normalized;
                emit themeModeChanged();
            }
            return;
        }
        sharedThemeMode = normalized;
        sharedThemeClients.insert(this);
        m_themeMode = normalized;
        clients = sharedThemeClients;
    }
    emit themeModeChanged();
    for (RuntimeClient *client : clients) {
        if (!client || client == this) {
            continue;
        }
        if (client->m_themeMode != normalized) {
            client->m_themeMode = normalized;
            emit client->themeModeChanged();
        }
    }
}

void RuntimeClient::setSessionId(const QString &value)
{
    const QString normalized = value.trimmed();
    if (normalized == m_sessionId) {
        return;
    }
    m_sessionId = normalized;
    emit sessionIdChanged();
}

QString RuntimeClient::connectionState() const
{
    return m_connectionState;
}

QString RuntimeClient::statusText() const
{
    return m_statusText;
}

QVariantList RuntimeClient::credentials() const
{
    return m_credentials;
}

QVariantList RuntimeClient::events() const
{
    return m_events;
}

QVariantList RuntimeClient::messages() const
{
    return m_messages;
}

QVariantList RuntimeClient::activities() const
{
    return m_activities;
}

QVariantList RuntimeClient::changedPaths() const
{
    return m_changedPaths;
}

QVariantMap RuntimeClient::diagnostics() const
{
    return m_diagnostics;
}

QVariantList RuntimeClient::models() const
{
    return m_models;
}

QVariantList RuntimeClient::projects() const
{
    return m_projects;
}

QVariantList RuntimeClient::sessions() const
{
    return m_sessions;
}

QVariantList RuntimeClient::checkpoints() const
{
    return m_checkpoints;
}

QVariantMap RuntimeClient::pendingApproval() const
{
    return m_pendingApproval;
}

void RuntimeClient::connectToRuntime()
{
    if (!m_runtimeHandle) {
        setConnectionState(QStringLiteral("connecting"), QStringLiteral("Starting local runtime..."));
        QString error;
        m_runtimeHandle = sharedRuntimeHandle(&error);
        if (!m_runtimeHandle) {
            setConnectionState(QStringLiteral("error"),
                               error.isEmpty() ? QStringLiteral("Suncode runtime could not be started") : error);
            return;
        }
        setBaseUrl(QStringLiteral("rust-sdk://local"));
    }
    setConnectionState(QStringLiteral("connecting"), QStringLiteral("Connecting to local runtime..."));
    {
        QMutexLocker locker(&sharedRuntimeMutex);
        sharedThemeClients.insert(this);
        if (m_themeMode != sharedThemeMode) {
            m_themeMode = sharedThemeMode;
            emit themeModeChanged();
        }
    }
    loadHealth();
    loadModels();
    loadSettings();
    loadCredentialStatus();
    loadProjects();
    refreshDiagnostics();
}

void RuntimeClient::refreshDiagnostics()
{
    requestJson(QStringLiteral("GET"), endpoint(QStringLiteral("/diagnostics")), {},
                [this](int, const QJsonObject &object) {
                    m_diagnostics = object.toVariantMap();
                    emit diagnosticsChanged();
                });
}

void RuntimeClient::loadCredentialStatus()
{
    requestJson(QStringLiteral("GET"), endpoint(QStringLiteral("/credentials")), {},
                [this](int, const QJsonObject &object) {
                    m_credentials = object.value(QStringLiteral("credentials")).toArray().toVariantList();
                    emit credentialsChanged();
                });
}

void RuntimeClient::saveCredential(const QString &provider, const QString &apiKey)
{
    const QString value = apiKey.trimmed();
    const QString normalizedProvider = provider.trimmed();
    if (value.isEmpty() || normalizedProvider.isEmpty()) return;
    requestJson(QStringLiteral("POST"), endpoint(QStringLiteral("/credentials/%1").arg(normalizedProvider)), {{QStringLiteral("api_key"), value}},
                [this](int, const QJsonObject &) {
                    loadCredentialStatus();
                    loadModels();
                    emit credentialStored();
                    setConnectionState(QStringLiteral("connected"), QStringLiteral("Credential stored"));
                });
}

void RuntimeClient::saveUserSetting(const QString &key, const QVariant &value)
{
    const QString normalized = key.trimmed();
    if (normalized.isEmpty()) {
        return;
    }
    requestJson(QStringLiteral("PUT"), endpoint(QStringLiteral("/settings")),
                {{QStringLiteral("scope"), QStringLiteral("user")},
                 {QStringLiteral("key"), normalized},
                 {QStringLiteral("value"), QJsonValue::fromVariant(value)}},
                [this, normalized, value](int, const QJsonObject &) {
                    if (normalized == QStringLiteral("theme_mode")) {
                        setThemeMode(value.toString());
                    } else if (normalized == QStringLiteral("default_model")) {
                        setSelectedModel(value.toString());
                    }
                    setConnectionState(QStringLiteral("connected"), QStringLiteral("Saved %1").arg(normalized));
                });
}

void RuntimeClient::removeCredential(const QString &provider)
{
    const QString normalizedProvider = provider.trimmed();
    if (normalizedProvider.isEmpty()) return;
    requestJson(QStringLiteral("DELETE"), endpoint(QStringLiteral("/credentials/%1").arg(normalizedProvider)), {},
                [this](int, const QJsonObject &) {
                    loadCredentialStatus();
                    loadModels();
                    setConnectionState(QStringLiteral("connected"), QStringLiteral("Credential removed"));
                });
}

void RuntimeClient::loadHealth()
{
    requestJson(QStringLiteral("GET"), endpoint(QStringLiteral("/health")), {},
                [this](int, const QJsonObject &) {
                    setConnectionState(QStringLiteral("connected"), QStringLiteral("Connected to local runtime"));
                });
}

void RuntimeClient::loadProjects()
{
    requestJson(QStringLiteral("GET"), endpoint(QStringLiteral("/projects")), {},
                [this](int, const QJsonObject &object) {
            m_projects = object.value(QStringLiteral("projects")).toArray().toVariantList();
            emit projectsChanged();
            if (m_autoSelectProject && m_projectId.isEmpty() && !m_projects.isEmpty()) {
                selectProject(m_projects.first().toMap().value(QStringLiteral("projectId")).toString());
            } else if (!m_projectId.isEmpty()) {
                loadSessions();
            }
        });
}

void RuntimeClient::openProject(const QString &path)
{
    QString value = path.trimmed();
    const QUrl candidate(value);
    if (candidate.isLocalFile()) {
        value = candidate.toLocalFile();
    }
    if (value.isEmpty()) {
        return;
    }
    requestJson(QStringLiteral("POST"), endpoint(QStringLiteral("/projects")),
                {{QStringLiteral("path"), value}},
                [this](int, const QJsonObject &object) {
                    setConnectionState(QStringLiteral("connected"), QStringLiteral("Project opened"));
                    const QString projectId = object.value(QStringLiteral("projectId")).toString();
                    setProjectId(projectId);
                    emit projectOpened(projectId);
                    loadProjects();
                });
}

void RuntimeClient::selectProject(const QString &value)
{
    setProjectId(value);
    if (!m_projectId.isEmpty()) {
        closeEventSubscription();
        clearSessionView();
        m_sessions.clear();
        emit sessionsChanged();
        requestJson(QStringLiteral("POST"), endpoint(QStringLiteral("/projects/%1/open").arg(QString::fromUtf8(QUrl::toPercentEncoding(m_projectId)))),
                    {}, [this](int, const QJsonObject &) {
                        setConnectionState(QStringLiteral("connected"), QStringLiteral("Project selected"));
                        loadSessions();
                    });
    }
}

void RuntimeClient::createSession(const QString &title)
{
    if (m_projectId.isEmpty()) {
        return;
    }
    QJsonObject body;
    const QString value = title.trimmed();
    if (!value.isEmpty()) {
        body.insert(QStringLiteral("title"), value);
    }
    body.insert(QStringLiteral("model"), m_selectedModel);
    requestJson(QStringLiteral("POST"), endpoint(QStringLiteral("/projects/%1/sessions").arg(QString::fromUtf8(QUrl::toPercentEncoding(m_projectId)))),
                body, [this](int, const QJsonObject &object) {
                    const QString createdSessionId = object.value(QStringLiteral("sessionId")).toString();
                    setSessionId(createdSessionId);
                    setConnectionState(QStringLiteral("connected"), QStringLiteral("Session created"));
                    loadSessions();
                });
}

void RuntimeClient::renameSession(const QString &title)
{
    renameSessionById(m_sessionId, title);
}

void RuntimeClient::renameSessionById(const QString &sessionId, const QString &title)
{
    const QString targetSessionId = sessionId.trimmed();
    const QString value = title.trimmed();
    if (targetSessionId.isEmpty() || value.isEmpty()) {
        return;
    }
    requestJson(QStringLiteral("PATCH"), endpoint(QStringLiteral("/sessions/%1").arg(QString::fromUtf8(QUrl::toPercentEncoding(targetSessionId)))),
                {{QStringLiteral("title"), value}}, [this](int, const QJsonObject &) {
                    setConnectionState(QStringLiteral("connected"), QStringLiteral("Session renamed"));
                    loadSessions();
                });
}

void RuntimeClient::archiveSession()
{
    archiveSessionById(m_sessionId);
}

void RuntimeClient::archiveSessionById(const QString &sessionId)
{
    const QString targetSessionId = sessionId.trimmed();
    if (targetSessionId.isEmpty()) {
        return;
    }
    requestJson(QStringLiteral("DELETE"), endpoint(QStringLiteral("/sessions/%1").arg(QString::fromUtf8(QUrl::toPercentEncoding(targetSessionId)))),
                {}, [this, targetSessionId](int, const QJsonObject &) {
                    if (targetSessionId == m_sessionId) {
                        closeEventSubscription();
                        clearSessionView();
                    }
                    setConnectionState(QStringLiteral("connected"), QStringLiteral("Session archived"));
                    loadSessions();
                });
}

void RuntimeClient::loadSessions()
{
    if (m_projectId.isEmpty()) {
        m_sessions.clear();
        emit sessionsChanged();
        return;
    }
    requestJson(QStringLiteral("GET"), endpoint(QStringLiteral("/projects/%1/sessions").arg(QString::fromUtf8(QUrl::toPercentEncoding(m_projectId)))),
                {}, [this](int, const QJsonObject &object) {
            m_sessions.clear();
            const QVariantList sessions = object.value(QStringLiteral("sessions")).toArray().toVariantList();
            qDebug() << "loaded sessions" << sessions.size() << "for project" << m_projectId;
            for (const QVariant &value : sessions) {
                QVariantMap session = value.toMap();
                if (session.value(QStringLiteral("title")).toString().isEmpty()) {
                    session.insert(QStringLiteral("title"), QStringLiteral("Untitled session"));
                }
                m_sessions.append(session);
            }
            emit sessionsChanged();
            QString selectedSession;
            QString selectedTitle;
            for (const QVariant &session : m_sessions) {
                const QVariantMap sessionMap = session.toMap();
                const QString sessionId = sessionMap.value(QStringLiteral("sessionId")).toString();
                if (sessionId == m_sessionId) {
                    selectedSession = sessionId;
                    selectedTitle = sessionMap.value(QStringLiteral("title")).toString();
                    break;
                }
            }
            if (selectedSession.isEmpty() && !m_sessions.isEmpty()) {
                const QVariantMap sessionMap = m_sessions.first().toMap();
                selectedSession = sessionMap.value(QStringLiteral("sessionId")).toString();
                selectedTitle = sessionMap.value(QStringLiteral("title")).toString();
            }
            if (!selectedSession.isEmpty()) {
                if (selectedTitle != m_sessionTitle) {
                    m_sessionTitle = selectedTitle.isEmpty() ? QStringLiteral("Untitled session") : selectedTitle;
                    emit sessionTitleChanged();
                }
                selectSession(selectedSession);
            } else {
                setSessionId({});
                if (!m_sessionTitle.isEmpty()) {
                    m_sessionTitle.clear();
                    emit sessionTitleChanged();
                }
                clearSessionView();
                m_checkpoints.clear();
                emit checkpointsChanged();
            }
        });
}

void RuntimeClient::loadCheckpoints()
{
    if (m_sessionId.isEmpty()) {
        m_checkpoints.clear();
        emit checkpointsChanged();
        return;
    }
    requestJson(QStringLiteral("GET"), endpoint(QStringLiteral("/sessions/%1/checkpoints").arg(QString::fromUtf8(QUrl::toPercentEncoding(m_sessionId)))),
                {}, [this](int, const QJsonObject &object) {
                    m_checkpoints = object.value(QStringLiteral("checkpoints")).toArray().toVariantList();
                    emit checkpointsChanged();
                });
}

void RuntimeClient::loadModels()
{
    requestJson(QStringLiteral("GET"), endpoint(QStringLiteral("/models")), {},
                [this](int, const QJsonObject &object) {
            const QJsonArray models = object.value(QStringLiteral("models")).toArray();
                    m_models = models.toVariantList();
                    emit modelsChanged();
                    if (!m_models.isEmpty() && m_selectedModel.isEmpty()) {
                        setSelectedModel(m_models.first().toMap().value(QStringLiteral("id")).toString());
                    }
        });
}

void RuntimeClient::loadSettings()
{
    requestJson(QStringLiteral("GET"), endpoint(QStringLiteral("/settings")), {},
                [this](int, const QJsonObject &object) {
                    const QJsonArray settings = object.value(QStringLiteral("settings")).toArray();
                    for (const QJsonValue &value : settings) {
                        const QJsonObject setting = value.toObject();
                        if (setting.value(QStringLiteral("key")).toString() == QStringLiteral("default_model")) {
                            const QString model = setting.value(QStringLiteral("value")).toString();
                            if (!model.isEmpty()) setSelectedModel(model);
                        } else if (setting.value(QStringLiteral("key")).toString() == QStringLiteral("theme_mode")) {
                            const QString mode = setting.value(QStringLiteral("value")).toString();
                            if (!mode.isEmpty()) setThemeMode(mode);
                        }
                    }
                });
}

void RuntimeClient::loadSession()
{
    clearSessionView();
    QUrlQuery query;
    query.addQueryItem(QStringLiteral("after"), QStringLiteral("0"));
    QUrl url = endpoint(QStringLiteral("/sessions/%1/snapshot").arg(QString::fromUtf8(QUrl::toPercentEncoding(m_sessionId))));
    url.setQuery(query);
    requestJson(QStringLiteral("GET"), url, {}, [this](int, const QJsonObject &object) {
        m_deferSessionReplaySignals = true;
        const QJsonArray messages = object.value(QStringLiteral("messages")).toArray();
        qDebug() << "snapshot messages" << messages.size() << "for session" << m_sessionId;
        for (qsizetype index = 0; index < messages.size(); ++index) {
            const QJsonObject message = messages.at(index).toObject();
            const QString role = message.value(QStringLiteral("role")).toString();
            if (role == QStringLiteral("user") || role == QStringLiteral("assistant")) {
                m_messages.append(messageFromJson(message, index + 1));
            }
        }
        const QJsonArray values = object.value(QStringLiteral("events")).toArray();
        for (const QJsonValue &value : values) {
            const QJsonObject event = value.toObject();
            QString eventType = event.value(QStringLiteral("event_type")).toString();
            if (eventType.isEmpty()) {
                eventType = event.value(QStringLiteral("eventType")).toString();
            }
            if (eventType == QStringLiteral("message.user")
                || eventType == QStringLiteral("message.assistant")
                || eventType == QStringLiteral("message.tool")) {
                continue;
            }
            consumeEvent(event);
        }
        const qint64 latestSequence = object.value(QStringLiteral("latest_sequence")).toInteger(
            object.value(QStringLiteral("latestSequence")).toInteger());
        if (latestSequence > m_lastSequence) {
            m_lastSequence = latestSequence;
        }
        m_deferSessionReplaySignals = false;
        emitReplayedSessionState();
        loadCheckpoints();
        startEventStream();
    });
}

void RuntimeClient::restoreCheckpoint(const QString &manifestId)
{
    const QString value = manifestId.trimmed();
    if (value.isEmpty() || m_sessionId.isEmpty()) {
        return;
    }
    requestJson(QStringLiteral("POST"), endpoint(QStringLiteral("/checkpoints/%1/restore").arg(QString::fromUtf8(QUrl::toPercentEncoding(value)))),
                {{QStringLiteral("session_id"), m_sessionId}}, [this](int, const QJsonObject &) {
                    setConnectionState(QStringLiteral("connected"), QStringLiteral("Turn changes restored"));
                    loadCheckpoints();
                });
}

void RuntimeClient::selectSession(const QString &value)
{
    setSessionId(value);
    closeEventSubscription();
    for (const QVariant &session : m_sessions) {
        const QVariantMap sessionMap = session.toMap();
        if (sessionMap.value(QStringLiteral("sessionId")).toString() == m_sessionId) {
            const QString title = sessionMap.value(QStringLiteral("title")).toString();
            if (title != m_sessionTitle) {
                m_sessionTitle = title.isEmpty() ? QStringLiteral("Untitled session") : title;
                emit sessionTitleChanged();
            }
            break;
        }
    }
    loadSession();
}

void RuntimeClient::startEventStream()
{
    closeEventSubscription();
    if (!m_runtimeHandle || m_sessionId.isEmpty()) {
        return;
    }
    char *error = nullptr;
    m_eventSubscription = suncode_runtime_sdk_subscribe_session(
        m_runtimeHandle,
        m_sessionId.toUtf8().constData(),
        m_lastSequence,
        &RuntimeClient::sdkEventCallback,
        this,
        &error);
    if (!m_eventSubscription) {
        const QString message = takeSdkString(error);
        setConnectionState(QStringLiteral("error"),
                           message.isEmpty() ? QStringLiteral("Session events could not be subscribed") : message);
    }
}

void RuntimeClient::sdkEventCallback(const char *eventJson, void *userData)
{
    auto *client = static_cast<RuntimeClient *>(userData);
    if (!client || !eventJson) {
        return;
    }
    const QJsonDocument document = QJsonDocument::fromJson(QByteArray(eventJson));
    if (!document.isObject()) {
        return;
    }
    const QJsonObject event = document.object();
    QMetaObject::invokeMethod(client, [client, event] {
        client->consumeEvent(event);
    }, Qt::QueuedConnection);
}

void RuntimeClient::closeEventSubscription()
{
    if (m_eventSubscription) {
        suncode_runtime_sdk_subscription_close(m_eventSubscription);
        m_eventSubscription = nullptr;
    }
}

void RuntimeClient::consumeEvent(const QJsonObject &event)
{
    const qint64 sequence = event.value(QStringLiteral("content_sequence")).toInteger(
        event.value(QStringLiteral("contentSequence")).toInteger());
    if (sequence > 0 && sequence <= m_lastSequence) {
        return;
    }
    if (sequence > 0) {
        m_lastSequence = sequence;
    }
    const QVariantMap value = mapFromJson(event);
    QVariantMap withText = value;
    if (!withText.contains(QStringLiteral("content_sequence"))) {
        withText.insert(QStringLiteral("content_sequence"), sequence);
    }
    QString eventType = event.value(QStringLiteral("event_type")).toString();
    if (eventType.isEmpty()) {
        eventType = event.value(QStringLiteral("eventType")).toString();
    }
    if (!withText.contains(QStringLiteral("event_type"))) {
        withText.insert(QStringLiteral("event_type"), eventType);
    }
    withText.insert(QStringLiteral("display_text"), eventText(value));
    m_events.append(withText);

    const QVariantMap payload = event.value(QStringLiteral("payload")).toObject().toVariantMap();
    bool messagesDirty = false;
    if (eventType == QStringLiteral("assistant.delta")) {
        const QString turnId = payload.value(QStringLiteral("turn_id")).toString();
        bool appended = false;
        for (qsizetype index = m_messages.size() - 1; index >= 0; --index) {
            QVariantMap message = m_messages.at(index).toMap();
            if (message.value(QStringLiteral("turn_id")).toString() == turnId
                && message.value(QStringLiteral("streaming")).toBool()) {
                message.insert(QStringLiteral("text"), message.value(QStringLiteral("text")).toString() + payload.value(QStringLiteral("text")).toString());
                message.insert(QStringLiteral("content_sequence"), sequence);
                m_messages[index] = message;
                appended = true;
                messagesDirty = true;
                break;
            }
        }
        if (!appended) {
            QVariantMap message;
            message.insert(QStringLiteral("role"), QStringLiteral("assistant"));
            message.insert(QStringLiteral("text"), payload.value(QStringLiteral("text")));
            message.insert(QStringLiteral("content_sequence"), sequence);
            message.insert(QStringLiteral("turn_id"), turnId);
            message.insert(QStringLiteral("streaming"), true);
            m_messages.append(message);
            messagesDirty = true;
        }
    } else if (eventType == QStringLiteral("message.user") || eventType == QStringLiteral("message.assistant")) {
        QVariantMap message;
        message.insert(QStringLiteral("role"), eventType == QStringLiteral("message.user") ? QStringLiteral("user") : QStringLiteral("assistant"));
        message.insert(QStringLiteral("text"), eventText(value));
        message.insert(QStringLiteral("content_sequence"), sequence);
        message.insert(QStringLiteral("turn_id"), payload.value(QStringLiteral("turn_id")));
        if (eventType == QStringLiteral("message.assistant")) {
            const QString turnId = payload.value(QStringLiteral("turn_id")).toString();
            for (qsizetype index = m_messages.size() - 1; index >= 0; --index) {
                const QVariantMap existing = m_messages.at(index).toMap();
                if (existing.value(QStringLiteral("turn_id")).toString() == turnId
                    && existing.value(QStringLiteral("streaming")).toBool()) {
                    m_messages.removeAt(index);
                }
            }
        }
        m_messages.append(message);
        messagesDirty = true;
    } else {
        QVariantMap activity;
        activity.insert(QStringLiteral("event_type"), eventType);
        activity.insert(QStringLiteral("text"), eventText(value));
        activity.insert(QStringLiteral("content_sequence"), sequence);
        activity.insert(QStringLiteral("state"), payload.value(QStringLiteral("state")));
        activity.insert(QStringLiteral("operation"), payload.value(QStringLiteral("operation")));
        activity.insert(QStringLiteral("path"), payload.value(QStringLiteral("path")));
        m_activities.append(activity);
    }

    const QStringList paths = {
        payload.value(QStringLiteral("path")).toString(),
        payload.value(QStringLiteral("from")).toString(),
        payload.value(QStringLiteral("to")).toString()
    };
    bool pathsChanged = false;
    for (const QString &path : paths) {
        if (!path.isEmpty() && !m_changedPaths.contains(path)) {
            m_changedPaths.append(path);
            pathsChanged = true;
        }
    }
    if (pathsChanged && !m_deferSessionReplaySignals) emit changedPathsChanged();
    if (messagesDirty && !m_deferSessionReplaySignals) emit messagesChanged();

    if (eventType == QStringLiteral("approval.requested")) {
        setPendingApproval(event.value(QStringLiteral("payload")).toObject().toVariantMap());
    }
    if (eventType == QStringLiteral("approval.resolved")) {
        setPendingApproval({});
    }
    if (eventType == QStringLiteral("turn.state")) {
        const QString state = payload.value(QStringLiteral("state")).toString();
        const QString turnId = payload.value(QStringLiteral("turn_id")).toString();
        const bool terminal = state == QStringLiteral("completed") || state == QStringLiteral("failed") || state == QStringLiteral("cancelled") || state == QStringLiteral("interrupted");
        const QString next = terminal ? QString() : turnId;
        if (next != m_activeTurnId) {
            m_activeTurnId = next;
            if (!m_deferSessionReplaySignals) {
                emit activeTurnChanged();
            }
        }
    }
    if (!m_deferSessionReplaySignals && eventType.startsWith(QStringLiteral("checkpoint."))) {
        loadCheckpoints();
    }
}

void RuntimeClient::emitReplayedSessionState()
{
    emit eventsChanged();
    emit messagesChanged();
    emit activitiesChanged();
    emit changedPathsChanged();
    emit pendingApprovalChanged();
    emit activeTurnChanged();
}

void RuntimeClient::submitTurn(const QString &input)
{
    const QString text = input.trimmed();
    if (text.isEmpty()) {
        return;
    }
    if (!isModelConfigured(m_selectedModel)) {
        return;
    }
    const QString idempotencyKey = QUuid::createUuid().toString(QUuid::WithoutBraces);
    requestJson(QStringLiteral("POST"), endpoint(QStringLiteral("/sessions/%1/turns").arg(QString::fromUtf8(QUrl::toPercentEncoding(m_sessionId)))),
                {{QStringLiteral("input"), text}, {QStringLiteral("idempotency_key"), idempotencyKey}, {QStringLiteral("model"), m_selectedModel}},
                [this](int status, const QJsonObject &object) {
                    if (status == 202) {
                        setConnectionState(QStringLiteral("connected"), QStringLiteral("Turn is awaiting approval"));
                        return;
                    }
                    setConnectionState(QStringLiteral("connected"), QStringLiteral("Turn submitted"));
                    Q_UNUSED(object);
                });
}

bool RuntimeClient::isModelConfigured(const QString &modelId) const
{
    for (const QVariant &value : m_models) {
        const QVariantMap model = value.toMap();
        if (model.value(QStringLiteral("id")).toString() != modelId) {
            continue;
        }
        return model.value(QStringLiteral("availability")).toString() == QStringLiteral("configured");
    }
    return false;
}

void RuntimeClient::cancelTurn()
{
    if (m_activeTurnId.isEmpty() || m_sessionId.isEmpty()) return;
    requestJson(QStringLiteral("POST"), endpoint(QStringLiteral("/sessions/%1/turns/%2/cancel").arg(QString::fromUtf8(QUrl::toPercentEncoding(m_sessionId)), QString::fromUtf8(QUrl::toPercentEncoding(m_activeTurnId)))),
                {}, [this](int, const QJsonObject &) {
                    setConnectionState(QStringLiteral("connected"), QStringLiteral("Cancellation requested"));
                });
}

void RuntimeClient::resolveApproval(const QString &decision)
{
    const QString approvalId = m_pendingApproval.value(QStringLiteral("approval_id")).toString();
    if (approvalId.isEmpty()) {
        return;
    }
    requestJson(QStringLiteral("POST"), endpoint(QStringLiteral("/approvals/%1").arg(QString::fromUtf8(QUrl::toPercentEncoding(approvalId)))),
                {{QStringLiteral("decision"), decision}},
                [this](int, const QJsonObject &) {
                    setConnectionState(QStringLiteral("connected"), QStringLiteral("Approval submitted"));
                });
}

void RuntimeClient::copyText(const QString &text)
{
    const QString value = text;
    if (value.isEmpty() || !QGuiApplication::clipboard()) {
        return;
    }
    QGuiApplication::clipboard()->setText(value);
}

void RuntimeClient::clearSessionView()
{
    m_events.clear();
    m_messages.clear();
    m_activities.clear();
    m_changedPaths.clear();
    m_lastSequence = 0;
    if (!m_activeTurnId.isEmpty()) {
        m_activeTurnId.clear();
        emit activeTurnChanged();
    }
    setPendingApproval({});
    emit eventsChanged();
    emit messagesChanged();
    emit activitiesChanged();
    emit changedPathsChanged();
}

QUrl RuntimeClient::endpoint(const QString &path) const
{
    return QUrl(path);
}

void RuntimeClient::requestJson(const QString &method, const QUrl &url, const QJsonObject &body,
                                std::function<void(int, const QJsonObject &)> onSuccess)
{
    if (!m_runtimeHandle) {
        setConnectionState(QStringLiteral("error"), QStringLiteral("Runtime SDK is not connected"));
        return;
    }
    const QByteArray methodBytes = method.toUtf8();
    const QByteArray pathBytes = sdkPath(url).toUtf8();
    const QByteArray bodyBytes = QJsonDocument(body).toJson(QJsonDocument::Compact);
    auto *watcher = new QFutureWatcher<RuntimeCallResult>(this);
    connect(watcher, &QFutureWatcher<RuntimeCallResult>::finished, this, [this, watcher, onSuccess] {
        const RuntimeCallResult result = watcher->result();
        watcher->deleteLater();
        if (!result.error.isEmpty()) {
            setConnectionState(QStringLiteral("error"), result.error);
            return;
        }
        if (result.status >= 200 && result.status < 300) {
            onSuccess(result.status, result.body);
            return;
        }
        QString message = result.body.value(QStringLiteral("message")).toString();
        if (message.isEmpty()) {
            message = result.body.value(QStringLiteral("error")).toObject().value(QStringLiteral("message")).toString();
        }
        if (message.isEmpty()) {
            message = QStringLiteral("Runtime request failed");
        }
        setConnectionState(QStringLiteral("error"), message);
    });
    watcher->setFuture(QtConcurrent::run([handle = m_runtimeHandle, methodBytes, pathBytes, bodyBytes] {
        RuntimeCallResult result;
        char *response = suncode_runtime_sdk_request_json(
            handle,
            methodBytes.constData(),
            pathBytes.constData(),
            bodyBytes.constData());
        const QJsonDocument document = QJsonDocument::fromJson(takeSdkString(response).toUtf8());
        if (!document.isObject()) {
            result.error = QStringLiteral("Runtime returned an invalid response");
            return result;
        }
        const QJsonObject envelope = document.object();
        result.status = envelope.value(QStringLiteral("status")).toInt();
        if (envelope.value(QStringLiteral("ok")).toBool()) {
            result.body = envelope.value(QStringLiteral("body")).toObject();
        } else {
            result.body = envelope.value(QStringLiteral("error")).toObject();
        }
        return result;
    }));
}

void RuntimeClient::setConnectionState(const QString &state, const QString &status)
{
    if (m_connectionState != state) {
        m_connectionState = state;
        emit connectionStateChanged();
    }
    if (m_statusText != status) {
        m_statusText = status;
        emit statusTextChanged();
    }
}

void RuntimeClient::setPendingApproval(const QVariantMap &approval)
{
    if (approval == m_pendingApproval) {
        return;
    }
    m_pendingApproval = approval;
    if (!m_deferSessionReplaySignals) {
        emit pendingApprovalChanged();
    }
}
