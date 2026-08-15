#include "runtimeclient.h"
#include "runtime_sdk.h"
#include "runtimeclient_support.h"

#include <QFutureWatcher>
#include <QGuiApplication>
#include <QJsonArray>
#include <QJsonDocument>
#include <QDebug>
#include <QClipboard>
#include <QMutex>
#include <QMutexLocker>
#include <QUuid>
#include <QtConcurrent/QtConcurrent>

#include <functional>

namespace {
QMutex sharedRuntimeMutex;
std::shared_ptr<SunCodeRuntimeHandle> sharedHandle;
QString sharedHandleError;
QString sharedThemeMode = QStringLiteral("dark");
QSet<RuntimeClient *> sharedThemeClients;
}

RuntimeClient::RuntimeClient(QObject *parent)
    : QObject(parent)
{
    m_gitRefreshTimer.setSingleShot(true);
    m_gitRefreshTimer.setInterval(180);
    connect(&m_gitRefreshTimer, &QTimer::timeout, this, &RuntimeClient::refreshGitStatus);
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

std::shared_ptr<SunCodeRuntimeHandle> RuntimeClient::sharedRuntimeHandle(QString *error)
{
    QMutexLocker locker(&sharedRuntimeMutex);
    if (!sharedHandle && sharedHandleError.isEmpty()) {
        if (suncode_runtime_sdk_abi_version() != SUNCODE_RUNTIME_SDK_ABI_VERSION) {
            sharedHandleError = QStringLiteral("Unsupported runtime SDK ABI version");
        }
    }
    if (!sharedHandle && sharedHandleError.isEmpty()) {
        char *sdkError = nullptr;
        SunCodeRuntimeHandle *handle = suncode_runtime_sdk_open_default(&sdkError);
        if (!handle) {
            sharedHandleError = takeSdkString(sdkError);
        } else {
            sharedHandle = std::shared_ptr<SunCodeRuntimeHandle>(
                handle, [](SunCodeRuntimeHandle *value) { suncode_runtime_sdk_close(value); });
        }
    }
    if (error) *error = sharedHandleError;
    return sharedHandle;
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
    clearGitView();
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

qint64 RuntimeClient::sessionTotalTokens() const
{
    return m_sessionTotalTokens;
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

QVariantMap RuntimeClient::gitStatus() const
{
    return m_gitStatus;
}

QVariantMap RuntimeClient::gitDiff() const
{
    return m_gitDiff;
}

QVariantList RuntimeClient::gitDiffRows() const
{
    return m_gitDiffRows;
}

QString RuntimeClient::gitState() const
{
    return m_gitState;
}

QString RuntimeClient::gitDiffState() const
{
    return m_gitDiffState;
}

QString RuntimeClient::gitError() const
{
    return m_gitError;
}

void RuntimeClient::connectToRuntime()
{
    if (!m_runtimeHandle) {
        setConnectionState(QStringLiteral("connecting"), QStringLiteral("Starting local runtime..."));
        QString error;
        m_runtimeHandle = sharedRuntimeHandle(&error);
        if (!m_runtimeHandle) {
            setConnectionState(QStringLiteral("error"),
                               error.isEmpty() ? QStringLiteral("SunCode runtime could not be started") : error);
            return;
        }
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
    requestSdk([handle = m_runtimeHandle] { return suncode_runtime_sdk_diagnostics(handle.get()); },
                [this](const QJsonObject &object) {
                    m_diagnostics = object.toVariantMap();
                    emit diagnosticsChanged();
                });
}

void RuntimeClient::refreshGitStatus()
{
    if (!m_runtimeHandle || m_projectId.isEmpty()) {
        clearGitView();
        return;
    }
    if (m_gitState != QStringLiteral("loading")) {
        m_gitState = QStringLiteral("loading");
        emit gitStateChanged();
    }
    if (!m_gitError.isEmpty()) {
        m_gitError.clear();
        emit gitErrorChanged();
    }
    const QString requestedProjectId = m_projectId;
    const QByteArray projectBytes = requestedProjectId.toUtf8();
    requestSdk([handle = m_runtimeHandle, projectBytes] {
                    return suncode_runtime_sdk_git_status(handle.get(), projectBytes.constData());
                }, [this, requestedProjectId](const QJsonObject &object) {
                    if (m_projectId != requestedProjectId) return;
                    m_gitStatus = object.toVariantMap();
                    m_gitState = QStringLiteral("ready");
                    emit gitStatusChanged();
                    emit gitStateChanged();
                }, [this, requestedProjectId](const QString &code, const QString &message) {
                    if (m_projectId != requestedProjectId) return;
                    m_gitStatus.clear();
                    m_gitState = code == QStringLiteral("not_git_repository")
                        ? QStringLiteral("not_repository")
                        : QStringLiteral("error");
                    m_gitError = message;
                    emit gitStatusChanged();
                    emit gitStateChanged();
                    emit gitErrorChanged();
                });
}

void RuntimeClient::loadGitDiff(const QString &scope, const QString &path)
{
    const QString normalizedScope = scope.trimmed().toLower();
    const QString normalizedPath = path.trimmed();
    if (m_projectId.isEmpty() || normalizedPath.isEmpty()
        || (normalizedScope != QStringLiteral("all")
            && normalizedScope != QStringLiteral("staged")
            && normalizedScope != QStringLiteral("unstaged"))) {
        return;
    }
    const QString requestKey = m_projectId + QLatin1Char('\n') + normalizedScope
        + QLatin1Char('\n') + normalizedPath;
    m_gitDiffRequestKey = requestKey;
    m_gitDiffState = QStringLiteral("loading");
    m_gitDiff.clear();
    m_gitDiffRows.clear();
    emit gitDiffChanged();
    emit gitDiffStateChanged();
    const QByteArray projectBytes = m_projectId.toUtf8();
    const QByteArray scopeBytes = normalizedScope.toUtf8();
    const QByteArray pathBytes = normalizedPath.toUtf8();
    requestSdk([handle = m_runtimeHandle, projectBytes, scopeBytes, pathBytes] {
                    return suncode_runtime_sdk_git_diff_file(
                        handle.get(), projectBytes.constData(), scopeBytes.constData(), pathBytes.constData());
                }, [this, requestKey](const QJsonObject &object) {
                    if (m_gitDiffRequestKey != requestKey) return;
                    m_gitDiff = object.toVariantMap();
                    m_gitDiffRows.clear();
                    const QJsonArray hunks = object.value(QStringLiteral("hunks")).toArray();
                    for (const QJsonValue &hunkValue : hunks) {
                        const QJsonObject hunk = hunkValue.toObject();
                        QVariantMap header;
                        header.insert(QStringLiteral("kind"), QStringLiteral("hunk"));
                        header.insert(QStringLiteral("text"), hunk.value(QStringLiteral("header")).toString());
                        m_gitDiffRows.append(header);
                        const QJsonArray lines = hunk.value(QStringLiteral("lines")).toArray();
                        for (const QJsonValue &line : lines) {
                            m_gitDiffRows.append(line.toObject().toVariantMap());
                        }
                    }
                    m_gitDiffState = QStringLiteral("ready");
                    emit gitDiffChanged();
                    emit gitDiffStateChanged();
                }, [this, requestKey](const QString &, const QString &message) {
                    if (m_gitDiffRequestKey != requestKey) return;
                    m_gitDiff.clear();
                    m_gitDiffRows.clear();
                    m_gitDiffState = QStringLiteral("error");
                    m_gitError = message;
                    emit gitDiffChanged();
                    emit gitDiffStateChanged();
                    emit gitErrorChanged();
                });
}

void RuntimeClient::loadCredentialStatus()
{
    requestSdk([handle = m_runtimeHandle] { return suncode_runtime_sdk_list_credentials(handle.get()); },
                [this](const QJsonObject &object) {
                    m_credentials = object.value(QStringLiteral("credentials")).toArray().toVariantList();
                    emit credentialsChanged();
                });
}

void RuntimeClient::saveCredential(const QString &provider, const QString &apiKey)
{
    const QString value = apiKey.trimmed();
    const QString normalizedProvider = provider.trimmed();
    if (value.isEmpty() || normalizedProvider.isEmpty()) return;
    const QByteArray providerBytes = normalizedProvider.toUtf8();
    const QByteArray keyBytes = value.toUtf8();
    requestSdk([handle = m_runtimeHandle, providerBytes, keyBytes] {
                    return suncode_runtime_sdk_set_credential(handle.get(), providerBytes.constData(), keyBytes.constData());
                }, [this](const QJsonObject &) {
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
    const QByteArray keyBytes = normalized.toUtf8();
    const QByteArray serializedValue = QJsonDocument(
        QJsonObject{{QStringLiteral("value"), QJsonValue::fromVariant(value)}}
    ).toJson(QJsonDocument::Compact);
    requestSdk([handle = m_runtimeHandle, keyBytes, serializedValue] {
                    return suncode_runtime_sdk_set_setting(handle.get(), "user", nullptr, nullptr,
                                                           keyBytes.constData(), serializedValue.constData());
                }, [this, normalized, value](const QJsonObject &) {
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
    const QByteArray providerBytes = normalizedProvider.toUtf8();
    requestSdk([handle = m_runtimeHandle, providerBytes] {
                    return suncode_runtime_sdk_remove_credential(handle.get(), providerBytes.constData());
                }, [this](const QJsonObject &) {
                    loadCredentialStatus();
                    loadModels();
                    setConnectionState(QStringLiteral("connected"), QStringLiteral("Credential removed"));
                });
}

void RuntimeClient::loadHealth()
{
    requestSdk([handle = m_runtimeHandle] { return suncode_runtime_sdk_health(handle.get()); },
                [this](const QJsonObject &) {
                    setConnectionState(QStringLiteral("connected"), QStringLiteral("Connected to local runtime"));
                });
}

void RuntimeClient::loadProjects()
{
    requestSdk([handle = m_runtimeHandle] { return suncode_runtime_sdk_list_projects(handle.get()); },
                [this](const QJsonObject &object) {
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
    const QByteArray pathBytes = value.toUtf8();
    requestSdk([handle = m_runtimeHandle, pathBytes] {
                    return suncode_runtime_sdk_open_project(handle.get(), pathBytes.constData(), nullptr);
                }, [this](const QJsonObject &object) {
                    setConnectionState(QStringLiteral("connected"), QStringLiteral("Project opened"));
                    const QString projectId = object.value(QStringLiteral("projectId")).toString();
                    setProjectId(projectId);
                    refreshGitStatus();
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
        const QByteArray projectBytes = m_projectId.toUtf8();
        requestSdk([handle = m_runtimeHandle, projectBytes] {
                        return suncode_runtime_sdk_select_project(handle.get(), projectBytes.constData());
                    }, [this](const QJsonObject &) {
                        setConnectionState(QStringLiteral("connected"), QStringLiteral("Project selected"));
                        refreshGitStatus();
                        loadSessions();
                    });
    }
}

void RuntimeClient::createSession(const QString &title)
{
    if (m_projectId.isEmpty()) {
        return;
    }
    const QString value = title.trimmed();
    const QByteArray projectBytes = m_projectId.toUtf8();
    const QByteArray titleBytes = value.toUtf8();
    const QByteArray modelBytes = m_selectedModel.toUtf8();
    requestSdk([handle = m_runtimeHandle, projectBytes, titleBytes, modelBytes] {
                    return suncode_runtime_sdk_create_session(
                        handle.get(), projectBytes.constData(), titleBytes.isEmpty() ? nullptr : titleBytes.constData(),
                        modelBytes.isEmpty() ? nullptr : modelBytes.constData());
                }, [this](const QJsonObject &object) {
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
    const QByteArray sessionBytes = targetSessionId.toUtf8();
    const QByteArray titleBytes = value.toUtf8();
    requestSdk([handle = m_runtimeHandle, sessionBytes, titleBytes] {
                    return suncode_runtime_sdk_rename_session(handle.get(), sessionBytes.constData(), titleBytes.constData());
                }, [this](const QJsonObject &) {
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
    const QByteArray sessionBytes = targetSessionId.toUtf8();
    requestSdk([handle = m_runtimeHandle, sessionBytes] {
                    return suncode_runtime_sdk_archive_session(handle.get(), sessionBytes.constData());
                }, [this, targetSessionId](const QJsonObject &) {
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
    const QByteArray projectBytes = m_projectId.toUtf8();
    requestSdk([handle = m_runtimeHandle, projectBytes] {
                    return suncode_runtime_sdk_list_sessions(handle.get(), projectBytes.constData());
                }, [this](const QJsonObject &object) {
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
    const QByteArray sessionBytes = m_sessionId.toUtf8();
    requestSdk([handle = m_runtimeHandle, sessionBytes] {
                    return suncode_runtime_sdk_list_checkpoints(handle.get(), sessionBytes.constData());
                }, [this](const QJsonObject &object) {
                    m_checkpoints = object.value(QStringLiteral("checkpoints")).toArray().toVariantList();
                    emit checkpointsChanged();
                });
}

void RuntimeClient::loadModels()
{
    requestSdk([handle = m_runtimeHandle] { return suncode_runtime_sdk_list_models(handle.get()); },
                [this](const QJsonObject &object) {
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
    requestSdk([handle = m_runtimeHandle] { return suncode_runtime_sdk_list_settings(handle.get(), nullptr, nullptr); },
                [this](const QJsonObject &object) {
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
    loadSessionUsage();
    const QByteArray sessionBytes = m_sessionId.toUtf8();
    requestSdk([handle = m_runtimeHandle, sessionBytes] {
        return suncode_runtime_sdk_session_snapshot(handle.get(), sessionBytes.constData(), 0);
    }, [this](const QJsonObject &object) {
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

void RuntimeClient::loadSessionUsage()
{
    if (m_sessionId.isEmpty()) {
        return;
    }
    const QString requestedSessionId = m_sessionId;
    const QByteArray sessionBytes = requestedSessionId.toUtf8();
    requestSdk([handle = m_runtimeHandle, sessionBytes] {
                    return suncode_runtime_sdk_session_usage(handle.get(), sessionBytes.constData());
                }, [this, requestedSessionId](const QJsonObject &object) {
                    if (m_sessionId != requestedSessionId) {
                        return;
                    }
                    const qint64 totalTokens = object.value(QStringLiteral("total_tokens")).toInteger();
                    if (totalTokens != m_sessionTotalTokens) {
                        m_sessionTotalTokens = totalTokens;
                        emit sessionUsageChanged();
                    }
                });
}

void RuntimeClient::restoreCheckpoint(const QString &manifestId)
{
    const QString value = manifestId.trimmed();
    if (value.isEmpty() || m_sessionId.isEmpty()) {
        return;
    }
    const QByteArray manifestBytes = value.toUtf8();
    const QByteArray sessionBytes = m_sessionId.toUtf8();
    requestSdk([handle = m_runtimeHandle, manifestBytes, sessionBytes] {
                    return suncode_runtime_sdk_restore_checkpoint(
                        handle.get(), manifestBytes.constData(), sessionBytes.constData());
                }, [this](const QJsonObject &) {
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
        m_runtimeHandle.get(),
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
    if (pathsChanged && !m_deferSessionReplaySignals) {
        emit changedPathsChanged();
        scheduleGitRefresh();
    }
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
        scheduleGitRefresh();
    }
    if (!m_deferSessionReplaySignals && eventType == QStringLiteral("usage.updated")) {
        loadSessionUsage();
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
    const QByteArray sessionBytes = m_sessionId.toUtf8();
    const QByteArray inputBytes = text.toUtf8();
    const QByteArray keyBytes = idempotencyKey.toUtf8();
    const QByteArray modelBytes = m_selectedModel.toUtf8();
    requestSdk([handle = m_runtimeHandle, sessionBytes, inputBytes, keyBytes, modelBytes] {
                    return suncode_runtime_sdk_submit_turn(
                        handle.get(), sessionBytes.constData(), inputBytes.constData(), keyBytes.constData(),
                        modelBytes.isEmpty() ? nullptr : modelBytes.constData());
                }, [this](const QJsonObject &object) {
                    const QString responseStatus = object.value(QStringLiteral("status")).toString();
                    if (responseStatus == QStringLiteral("queued")) {
                        setConnectionState(QStringLiteral("connected"), QStringLiteral("Message queued for this turn"));
                        return;
                    }
                    if (responseStatus == QStringLiteral("awaiting_approval")) {
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
    const QByteArray sessionBytes = m_sessionId.toUtf8();
    const QByteArray turnBytes = m_activeTurnId.toUtf8();
    requestSdk([handle = m_runtimeHandle, sessionBytes, turnBytes] {
                    return suncode_runtime_sdk_cancel_turn(handle.get(), sessionBytes.constData(), turnBytes.constData());
                }, [this](const QJsonObject &) {
                    setConnectionState(QStringLiteral("connected"), QStringLiteral("Cancellation requested"));
                });
}

void RuntimeClient::resolveApproval(const QString &decision)
{
    const QString approvalId = m_pendingApproval.value(QStringLiteral("approval_id")).toString();
    if (approvalId.isEmpty()) {
        return;
    }
    const QByteArray approvalBytes = approvalId.toUtf8();
    const QByteArray decisionBytes = decision.toUtf8();
    requestSdk([handle = m_runtimeHandle, approvalBytes, decisionBytes] {
                    return suncode_runtime_sdk_resolve_approval(
                        handle.get(), approvalBytes.constData(), decisionBytes.constData());
                }, [this](const QJsonObject &) {
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
    if (m_sessionTotalTokens != 0) {
        m_sessionTotalTokens = 0;
        emit sessionUsageChanged();
    }
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

void RuntimeClient::scheduleGitRefresh()
{
    if (!m_projectId.isEmpty()) {
        m_gitRefreshTimer.start();
    }
}

void RuntimeClient::clearGitView()
{
    m_gitRefreshTimer.stop();
    m_gitStatus.clear();
    m_gitDiff.clear();
    m_gitDiffRows.clear();
    m_gitDiffRequestKey.clear();
    m_gitState = QStringLiteral("idle");
    m_gitDiffState = QStringLiteral("idle");
    m_gitError.clear();
    emit gitStatusChanged();
    emit gitDiffChanged();
    emit gitStateChanged();
    emit gitDiffStateChanged();
    emit gitErrorChanged();
}

void RuntimeClient::requestSdk(std::function<char *()> call,
                               std::function<void(const QJsonObject &)> onSuccess,
                               std::function<void(const QString &, const QString &)> onError)
{
    if (!m_runtimeHandle) {
        if (onError) {
            onError(QStringLiteral("runtime_unavailable"), QStringLiteral("Runtime SDK is not connected"));
        } else {
            setConnectionState(QStringLiteral("error"), QStringLiteral("Runtime SDK is not connected"));
        }
        return;
    }
    auto *watcher = new QFutureWatcher<RuntimeCallResult>(this);
    connect(watcher, &QFutureWatcher<RuntimeCallResult>::finished, this, [this, watcher, onSuccess, onError] {
        const RuntimeCallResult result = watcher->result();
        watcher->deleteLater();
        if (!result.error.isEmpty()) {
            if (onError) {
                onError(result.errorCode, result.error);
            } else {
                setConnectionState(QStringLiteral("error"), result.error);
            }
            return;
        }
        onSuccess(result.body);
    });
    watcher->setFuture(QtConcurrent::run([call = std::move(call)] {
        RuntimeCallResult result;
        char *response = call();
        const QJsonDocument document = QJsonDocument::fromJson(takeSdkString(response).toUtf8());
        if (!document.isObject()) {
            result.error = QStringLiteral("Runtime returned an invalid response");
            return result;
        }
        const QJsonObject envelope = document.object();
        if (envelope.value(QStringLiteral("ok")).toBool()) {
            result.body = envelope.value(QStringLiteral("body")).toObject();
        } else {
            const QJsonObject error = envelope.value(QStringLiteral("error")).toObject();
            result.errorCode = error.value(QStringLiteral("code")).toString();
            result.error = error.value(QStringLiteral("message")).toString();
            if (result.error.isEmpty()) {
                result.error = QStringLiteral("Runtime SDK call failed");
            }
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
