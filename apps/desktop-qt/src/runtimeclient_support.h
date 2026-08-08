#pragma once

#include <QJsonObject>
#include <QUrl>
#include <QVariantMap>

struct RuntimeCallResult {
    int status = 0;
    QJsonObject body;
    QString error;
};

QString takeSdkString(char *value);
QString sdkPath(const QUrl &url);
QVariantMap mapFromJson(const QJsonObject &object);
QString eventText(const QVariantMap &event);
QVariantMap messageFromJson(const QJsonObject &message, qint64 fallbackSequence = 0);
