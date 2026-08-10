import QtQuick
import QtQuick.Controls
import "../components"

Button {
    id: control
    property var theme
    property bool locked: true

    implicitWidth: 26
    implicitHeight: 26
    hoverEnabled: true
    focusPolicy: Qt.TabFocus
    Accessible.name: locked ? "Unlock sidebar" : "Lock sidebar open"

    background: Rectangle {
        radius: theme.radiusSmall
        color: control.hovered ? theme.surfaceHover : "transparent"
        border.width: control.visualFocus ? 2 : 0
        border.color: theme.accent
    }

    contentItem: ThemeIcon {
        anchors.centerIn: parent
        width: 16
        height: 16
        source: control.locked ? "qrc:/assets/icons/lock.svg" : "qrc:/assets/icons/unlock.svg"
        color: control.locked ? theme.accent : theme.textMuted
    }
}
