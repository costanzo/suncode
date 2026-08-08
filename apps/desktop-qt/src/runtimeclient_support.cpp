#include "runtimeclient_support.h"

#include "runtime_sdk.h"

#include <QJsonArray>

QString takeSdkString(char *value)
{
    if (!value) {
        return {};
    }
    const QString result = QString::fromUtf8(value);
    suncode_runtime_sdk_string_free(value);
    return result;
}

QString sdkPath(const QUrl &url)
{
    QString value = url.toString(QUrl::FullyEncoded);
    if (value.isEmpty()) {
        value = url.path(QUrl::FullyEncoded);
    }
    return value;
}

QVariantMap mapFromJson(const QJsonObject &object)
{
    return object.toVariantMap();
}

QString eventText(const QVariantMap &event)
{
    QString eventType = event.value(QStringLiteral("event_type")).toString();
    if (eventType.isEmpty()) {
        eventType = event.value(QStringLiteral("eventType")).toString();
    }
    const QVariantMap payload = event.value(QStringLiteral("payload")).toMap();
    const QVariantMap message = payload.value(QStringLiteral("message")).toMap();
    const QVariantList content = message.value(QStringLiteral("content")).toList();
    if (!content.isEmpty()) {
        const QVariantMap part = content.first().toMap();
        if (part.value(QStringLiteral("type")).toString() == QStringLiteral("text")) {
            return part.value(QStringLiteral("text")).toString();
        }
    }
    if (eventType == QStringLiteral("approval.requested")) {
        return QStringLiteral("Approval required for %1").arg(payload.value(QStringLiteral("operation")).toString());
    }
    if (eventType == QStringLiteral("checkpoint.captured")) {
        return QStringLiteral("Checkpoint captured for %1").arg(payload.value(QStringLiteral("path")).toString());
    }
    if (eventType == QStringLiteral("checkpoint.restore_failed")) {
        return QStringLiteral("Undo stopped because a file changed outside Suncode");
    }
    if (eventType == QStringLiteral("turn.state")) {
        return QStringLiteral("Turn %1").arg(payload.value(QStringLiteral("state")).toString());
    }
    if (eventType == QStringLiteral("assistant.delta")) {
        return payload.value(QStringLiteral("text")).toString();
    }
    return eventType;
}

QVariantMap messageFromJson(const QJsonObject &message, qint64 fallbackSequence)
{
    QVariantMap result;
    const QJsonArray content = message.value(QStringLiteral("content")).toArray();
    QString text;
    if (!content.isEmpty()) {
        const QJsonObject part = content.first().toObject();
        if (part.value(QStringLiteral("type")).toString() == QStringLiteral("text")) {
            text = part.value(QStringLiteral("text")).toString();
        }
    }
    result.insert(QStringLiteral("role"), message.value(QStringLiteral("role")).toString());
    result.insert(QStringLiteral("text"), text);
    result.insert(QStringLiteral("content_sequence"), fallbackSequence);
    return result;
}
