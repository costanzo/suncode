#pragma once

#include <QJsonObject>
#include <QVariantMap>

struct RuntimeCallResult {
    QJsonObject body;
    QString errorCode;
    QString error;
};

QString takeSdkString(char *value);
QVariantMap mapFromJson(const QJsonObject &object);
QString eventText(const QVariantMap &event);
QVariantMap messageFromJson(const QJsonObject &message, qint64 fallbackSequence = 0);
