#include "runtimeclient.h"

#include <QGuiApplication>
#include <QFont>
#include <QFontDatabase>
#include <QIcon>
#include <QQmlContext>
#include <QQmlApplicationEngine>
#include <QQuickStyle>
#include <QStringList>
#include <QUrl>
#include <QtQml/qqml.h>

namespace {

QString resolveFontFamily(const QStringList &candidateFamilies)
{
    const QStringList availableFamilies = QFontDatabase::families();
    for (const QString &candidateFamily : candidateFamilies) {
        if (candidateFamily.isEmpty()) {
            continue;
        }
        for (const QString &availableFamily : availableFamilies) {
            if (availableFamily.compare(candidateFamily, Qt::CaseInsensitive) == 0) {
                return availableFamily;
            }
        }
    }
    return availableFamilies.isEmpty() ? QStringLiteral("Arial") : availableFamilies.first();
}

}

int main(int argc, char *argv[])
{
    QQuickStyle::setStyle(QStringLiteral("Fusion"));

    QGuiApplication app(argc, argv);
    app.setApplicationDisplayName(QStringLiteral("SunCode"));
    app.setApplicationName(QStringLiteral("SunCode"));
    app.setOrganizationName(QStringLiteral("SunCode"));
    app.setWindowIcon(QIcon(QStringLiteral(":/assets/logo/suncode-logo-small-256.png")));

    const QString systemUiFont = app.font().family();
    const QString systemMonoFont = QFontDatabase::systemFont(QFontDatabase::FixedFont).family();
    const QString uiFont = resolveFontFamily(QStringList{
        QStringLiteral("Noto Sans"),
        QStringLiteral("Helvetica Neue"),
        QStringLiteral("Arial"),
        QStringLiteral("Verdana"),
        systemUiFont
    });
    const QString cjkFont = resolveFontFamily(QStringList{
        QStringLiteral("Noto Sans CJK SC"),
        QStringLiteral("Noto Sans SC"),
        QStringLiteral("PingFang SC"),
        QStringLiteral("Heiti SC"),
        uiFont
    });
    const QString monoFont = resolveFontFamily(QStringList{
        QStringLiteral("JetBrains Mono"),
        QStringLiteral("SF Mono"),
        QStringLiteral("Menlo"),
        QStringLiteral("Monaco"),
        QStringLiteral("Consolas"),
        systemMonoFont
    });

    QFont appFont = app.font();
    appFont.setStyleHint(QFont::SansSerif);
    QStringList uiFontStack{uiFont};
    if (cjkFont.compare(uiFont, Qt::CaseInsensitive) != 0) {
        uiFontStack.append(cjkFont);
    }
    appFont.setFamilies(uiFontStack);
    app.setFont(appFont);

    qmlRegisterType<RuntimeClient>("SunCode.Runtime", 1, 0, "RuntimeClient");

    QQmlApplicationEngine engine;
    engine.rootContext()->setContextProperty(QStringLiteral("SunCodeUiFontFamily"), uiFont);
    engine.rootContext()->setContextProperty(QStringLiteral("SunCodeCjkFontFamily"), cjkFont);
    engine.rootContext()->setContextProperty(QStringLiteral("SunCodeMonoFontFamily"), monoFont);
    const QUrl url(QStringLiteral("qrc:/qt/qml/SunCode/Desktop/qml/app/ProjectHub.qml"));
    QObject::connect(&engine, &QQmlApplicationEngine::objectCreationFailed,
                     &app, [] { QCoreApplication::exit(EXIT_FAILURE); }, Qt::QueuedConnection);
    engine.load(url);
    if (engine.rootObjects().isEmpty()) {
        return EXIT_FAILURE;
    }
    return app.exec();
}
