#pragma once

#include <QJsonObject>
#include <QObject>
#include <QTimer>
#include <QUrl>
#include <QVariantList>
#include <QVariantMap>

#include "runtime_sdk.h"

#include <functional>

class RuntimeClient : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString baseUrl READ baseUrl WRITE setBaseUrl NOTIFY baseUrlChanged)
    Q_PROPERTY(QString projectId READ projectId WRITE setProjectId NOTIFY projectIdChanged)
    Q_PROPERTY(QString sessionId READ sessionId WRITE setSessionId NOTIFY sessionIdChanged)
    Q_PROPERTY(QString sessionTitle READ sessionTitle NOTIFY sessionTitleChanged)
    Q_PROPERTY(QString activeTurnId READ activeTurnId NOTIFY activeTurnChanged)
    Q_PROPERTY(QString selectedModel READ selectedModel WRITE setSelectedModel NOTIFY selectedModelChanged)
    Q_PROPERTY(QString themeMode READ themeMode WRITE setThemeMode NOTIFY themeModeChanged)
    Q_PROPERTY(bool autoSelectProject READ autoSelectProject WRITE setAutoSelectProject)
    Q_PROPERTY(QString connectionState READ connectionState NOTIFY connectionStateChanged)
    Q_PROPERTY(QString statusText READ statusText NOTIFY statusTextChanged)
    Q_PROPERTY(bool deepSeekConfigured READ deepSeekConfigured NOTIFY deepSeekConfiguredChanged)
    Q_PROPERTY(QVariantList events READ events NOTIFY eventsChanged)
    Q_PROPERTY(QVariantList messages READ messages NOTIFY messagesChanged)
    Q_PROPERTY(QVariantList activities READ activities NOTIFY activitiesChanged)
    Q_PROPERTY(QVariantList changedPaths READ changedPaths NOTIFY changedPathsChanged)
    Q_PROPERTY(QVariantMap diagnostics READ diagnostics NOTIFY diagnosticsChanged)
    Q_PROPERTY(QVariantList models READ models NOTIFY modelsChanged)
    Q_PROPERTY(QVariantList projects READ projects NOTIFY projectsChanged)
    Q_PROPERTY(QVariantList sessions READ sessions NOTIFY sessionsChanged)
    Q_PROPERTY(QVariantList checkpoints READ checkpoints NOTIFY checkpointsChanged)
    Q_PROPERTY(QVariantMap pendingApproval READ pendingApproval NOTIFY pendingApprovalChanged)

public:
    explicit RuntimeClient(QObject *parent = nullptr);
    ~RuntimeClient() override;

    QString baseUrl() const;
    void setBaseUrl(const QString &value);
    QString projectId() const;
    void setProjectId(const QString &value);
    QString sessionId() const;
    QString sessionTitle() const;
    void setSessionId(const QString &value);
    QString activeTurnId() const;
    QString selectedModel() const;
    void setSelectedModel(const QString &value);
    QString themeMode() const;
    void setThemeMode(const QString &value);
    bool autoSelectProject() const { return m_autoSelectProject; }
    void setAutoSelectProject(bool value) { m_autoSelectProject = value; }
    QString connectionState() const;
    QString statusText() const;
    bool deepSeekConfigured() const;
    QVariantList events() const;
    QVariantList messages() const;
    QVariantList activities() const;
    QVariantList changedPaths() const;
    QVariantMap diagnostics() const;
    QVariantList models() const;
    QVariantList projects() const;
    QVariantList sessions() const;
    QVariantList checkpoints() const;
    QVariantMap pendingApproval() const;

    Q_INVOKABLE void connectToRuntime();
    Q_INVOKABLE void loadCredentialStatus();
    Q_INVOKABLE void saveDeepSeekApiKey(const QString &apiKey);
    Q_INVOKABLE void removeDeepSeekApiKey();
    Q_INVOKABLE void loadProjects();
    Q_INVOKABLE void openProject(const QString &path);
    Q_INVOKABLE void selectProject(const QString &value);
    Q_INVOKABLE void createSession(const QString &title);
    Q_INVOKABLE void renameSession(const QString &title);
    Q_INVOKABLE void archiveSession();
    Q_INVOKABLE void restoreCheckpoint(const QString &manifestId);
    Q_INVOKABLE void loadSession();
    Q_INVOKABLE void selectSession(const QString &value);
    Q_INVOKABLE void submitTurn(const QString &input);
    Q_INVOKABLE void cancelTurn();
    Q_INVOKABLE void resolveApproval(const QString &decision);
    Q_INVOKABLE void copyText(const QString &text);
    Q_INVOKABLE void clearSessionView();
    Q_INVOKABLE void refreshDiagnostics();
    Q_INVOKABLE void saveUserSetting(const QString &key, const QVariant &value);

signals:
    void baseUrlChanged();
    void projectIdChanged();
    void sessionIdChanged();
    void sessionTitleChanged();
    void activeTurnChanged();
    void selectedModelChanged();
    void themeModeChanged();
    void connectionStateChanged();
    void statusTextChanged();
    void deepSeekConfiguredChanged();
    void eventsChanged();
    void messagesChanged();
    void activitiesChanged();
    void changedPathsChanged();
    void diagnosticsChanged();
    void modelsChanged();
    void projectsChanged();
    void sessionsChanged();
    void checkpointsChanged();
    void pendingApprovalChanged();
    void credentialStored();
    void projectOpened(const QString &projectId);

private:
    QUrl endpoint(const QString &path) const;
    void setConnectionState(const QString &state, const QString &status);
    void requestJson(const QString &method, const QUrl &url, const QJsonObject &body,
                     std::function<void(int, const QJsonObject &)> onSuccess);
    void loadHealth();
    void loadModels();
    void loadSettings();
    void loadSessions();
    void loadCheckpoints();
    void startEventStream();
    void consumeEvent(const QJsonObject &event);
    void emitReplayedSessionState();
    void setPendingApproval(const QVariantMap &approval);
    void closeEventSubscription();
    static void sdkEventCallback(const char *eventJson, void *userData);

    static SuncodeRuntimeHandle *sharedRuntimeHandle(QString *error);
    SuncodeRuntimeHandle *m_runtimeHandle = nullptr;
    SuncodeRuntimeSubscriptionHandle *m_eventSubscription = nullptr;
    QString m_baseUrl;
    QString m_projectId;
    QString m_sessionId;
    QString m_sessionTitle;
    QString m_activeTurnId;
    QString m_selectedModel = QStringLiteral("deepseek-v4-flash");
    QString m_themeMode = QStringLiteral("dark");
    QString m_connectionState = QStringLiteral("disconnected");
    QString m_statusText = QStringLiteral("Not connected");
    QVariantList m_events;
    QVariantList m_messages;
    QVariantList m_activities;
    QVariantList m_changedPaths;
    QVariantMap m_diagnostics;
    QVariantList m_models;
    QVariantList m_projects;
    QVariantList m_sessions;
    QVariantList m_checkpoints;
    QVariantMap m_pendingApproval;
    bool m_deepSeekConfigured = false;
    bool m_autoSelectProject = true;
    bool m_deferSessionReplaySignals = false;
    qint64 m_lastSequence = 0;
};
