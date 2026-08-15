#pragma once

#include <QJsonObject>
#include <QObject>
#include <QTimer>
#include <QUrl>
#include <QVariantList>
#include <QVariantMap>

#include "runtime_sdk.h"

#include <functional>
#include <memory>

class RuntimeClient : public QObject {
    Q_OBJECT
    Q_PROPERTY(QString projectId READ projectId WRITE setProjectId NOTIFY projectIdChanged)
    Q_PROPERTY(QString sessionId READ sessionId WRITE setSessionId NOTIFY sessionIdChanged)
    Q_PROPERTY(QString sessionTitle READ sessionTitle NOTIFY sessionTitleChanged)
    Q_PROPERTY(QString activeTurnId READ activeTurnId NOTIFY activeTurnChanged)
    Q_PROPERTY(QString selectedModel READ selectedModel WRITE setSelectedModel NOTIFY selectedModelChanged)
    Q_PROPERTY(qint64 sessionTotalTokens READ sessionTotalTokens NOTIFY sessionUsageChanged)
    Q_PROPERTY(QString themeMode READ themeMode WRITE setThemeMode NOTIFY themeModeChanged)
    Q_PROPERTY(bool autoSelectProject READ autoSelectProject WRITE setAutoSelectProject)
    Q_PROPERTY(QString connectionState READ connectionState NOTIFY connectionStateChanged)
    Q_PROPERTY(QString statusText READ statusText NOTIFY statusTextChanged)
    Q_PROPERTY(QVariantList credentials READ credentials NOTIFY credentialsChanged)
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
    Q_PROPERTY(QVariantMap gitStatus READ gitStatus NOTIFY gitStatusChanged)
    Q_PROPERTY(QVariantMap gitDiff READ gitDiff NOTIFY gitDiffChanged)
    Q_PROPERTY(QVariantList gitDiffRows READ gitDiffRows NOTIFY gitDiffChanged)
    Q_PROPERTY(QString gitState READ gitState NOTIFY gitStateChanged)
    Q_PROPERTY(QString gitDiffState READ gitDiffState NOTIFY gitDiffStateChanged)
    Q_PROPERTY(QString gitError READ gitError NOTIFY gitErrorChanged)

public:
    explicit RuntimeClient(QObject *parent = nullptr);
    ~RuntimeClient() override;

    QString projectId() const;
    void setProjectId(const QString &value);
    QString sessionId() const;
    QString sessionTitle() const;
    void setSessionId(const QString &value);
    QString activeTurnId() const;
    QString selectedModel() const;
    void setSelectedModel(const QString &value);
    qint64 sessionTotalTokens() const;
    QString themeMode() const;
    void setThemeMode(const QString &value);
    bool autoSelectProject() const { return m_autoSelectProject; }
    void setAutoSelectProject(bool value) { m_autoSelectProject = value; }
    QString connectionState() const;
    QString statusText() const;
    QVariantList credentials() const;
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
    QVariantMap gitStatus() const;
    QVariantMap gitDiff() const;
    QVariantList gitDiffRows() const;
    QString gitState() const;
    QString gitDiffState() const;
    QString gitError() const;

    Q_INVOKABLE void connectToRuntime();
    Q_INVOKABLE void loadCredentialStatus();
    Q_INVOKABLE void saveCredential(const QString &provider, const QString &apiKey);
    Q_INVOKABLE void removeCredential(const QString &provider);
    Q_INVOKABLE void loadProjects();
    Q_INVOKABLE void openProject(const QString &path);
    Q_INVOKABLE void selectProject(const QString &value);
    Q_INVOKABLE void createSession(const QString &title);
    Q_INVOKABLE void renameSession(const QString &title);
    Q_INVOKABLE void renameSessionById(const QString &sessionId, const QString &title);
    Q_INVOKABLE void archiveSession();
    Q_INVOKABLE void archiveSessionById(const QString &sessionId);
    Q_INVOKABLE void restoreCheckpoint(const QString &manifestId);
    Q_INVOKABLE void loadSession();
    Q_INVOKABLE void selectSession(const QString &value);
    Q_INVOKABLE void submitTurn(const QString &input);
    Q_INVOKABLE void cancelTurn();
    Q_INVOKABLE void resolveApproval(const QString &decision);
    Q_INVOKABLE void copyText(const QString &text);
    Q_INVOKABLE void clearSessionView();
    Q_INVOKABLE void refreshDiagnostics();
    Q_INVOKABLE void refreshGitStatus();
    Q_INVOKABLE void loadGitDiff(const QString &scope, const QString &path);
    Q_INVOKABLE void saveUserSetting(const QString &key, const QVariant &value);

signals:
    void projectIdChanged();
    void sessionIdChanged();
    void sessionTitleChanged();
    void activeTurnChanged();
    void selectedModelChanged();
    void sessionUsageChanged();
    void themeModeChanged();
    void connectionStateChanged();
    void statusTextChanged();
    void credentialsChanged();
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
    void gitStatusChanged();
    void gitDiffChanged();
    void gitStateChanged();
    void gitDiffStateChanged();
    void gitErrorChanged();
    void credentialStored();
    void projectOpened(const QString &projectId);

private:
    void setConnectionState(const QString &state, const QString &status);
    void requestSdk(std::function<char *()> call,
                    std::function<void(const QJsonObject &)> onSuccess,
                    std::function<void(const QString &, const QString &)> onError = {});
    bool isModelConfigured(const QString &modelId) const;
    void loadHealth();
    void loadModels();
    void loadSettings();
    void loadSessions();
    void loadCheckpoints();
    void loadSessionUsage();
    void scheduleGitRefresh();
    void clearGitView();
    void startEventStream();
    void consumeEvent(const QJsonObject &event);
    void emitReplayedSessionState();
    void setPendingApproval(const QVariantMap &approval);
    void closeEventSubscription();
    static void sdkEventCallback(const char *eventJson, void *userData);

    static std::shared_ptr<SunCodeRuntimeHandle> sharedRuntimeHandle(QString *error);
    std::shared_ptr<SunCodeRuntimeHandle> m_runtimeHandle;
    SunCodeRuntimeSubscriptionHandle *m_eventSubscription = nullptr;
    QString m_projectId;
    QString m_sessionId;
    QString m_sessionTitle;
    QString m_activeTurnId;
    QString m_selectedModel = QStringLiteral("deepseek-v4-flash");
    qint64 m_sessionTotalTokens = 0;
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
    QVariantMap m_gitStatus;
    QVariantMap m_gitDiff;
    QVariantList m_gitDiffRows;
    QString m_gitState = QStringLiteral("idle");
    QString m_gitDiffState = QStringLiteral("idle");
    QString m_gitError;
    QString m_gitDiffRequestKey;
    QVariantList m_credentials;
    QTimer m_gitRefreshTimer;
    bool m_autoSelectProject = true;
    bool m_deferSessionReplaySignals = false;
    qint64 m_lastSequence = 0;
};
