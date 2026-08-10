import QtQuick
import QtQuick.Controls
import "../components"

Button {
    id: control
    property var theme
    property string side: "left"

    implicitWidth: 28
    implicitHeight: 28
    hoverEnabled: true
    focusPolicy: Qt.TabFocus

    background: Rectangle {
        radius: theme.radiusSmall
        color: control.checked ? theme.surfaceActive : (control.hovered ? theme.surfaceHover : "transparent")
        border.width: control.visualFocus ? 2 : 1
        border.color: control.visualFocus ? theme.accent : (control.checked ? theme.borderStrong : theme.border)
    }

    contentItem: ThemeIcon {
        anchors.centerIn: parent
        width: 18
        height: 18
        source: control.side === "right"
                ? "qrc:/assets/icons/panel-right.svg"
                : "qrc:/assets/icons/panel-left.svg"
        color: control.checked ? theme.text : theme.textSecondary
    }
}
