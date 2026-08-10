import QtQuick
import QtQuick.Controls

Button {
    id: control
    property var theme
    property string tone: "neutral"
    property bool compact: false

    hoverEnabled: true
    implicitHeight: compact ? theme.compactControlHeight : theme.controlHeight
    implicitWidth: Math.max(compact ? 64 : 84, label.implicitWidth + (compact ? 20 : 28))
    leftPadding: compact ? 10 : 14
    rightPadding: compact ? 10 : 14

    contentItem: Text {
        id: label
        text: control.text
        color: {
            if (!control.enabled) return theme.textDisabled
            if (control.tone === "primary") return theme.accentInk
            if (control.tone === "danger") return theme.danger
            return theme.text
        }
        font.family: theme.fontUi
        font.pixelSize: theme.typeLabel
        font.weight: control.tone === "primary" ? Font.DemiBold : Font.Medium
        horizontalAlignment: Text.AlignHCenter
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
    }

    background: Rectangle {
        radius: theme.radiusSmall
        color: {
            if (!control.enabled) return theme.surface
            if (control.tone === "primary") {
                if (control.down) return theme.accentPressed
                return control.hovered ? theme.accentHover : theme.accent
            }
            if (control.tone === "danger") {
                if (control.down || control.hovered) return theme.dangerSurface
                return "transparent"
            }
            if (control.checked || control.down) return theme.surfaceActive
            return control.hovered ? theme.surfaceHover : theme.surfaceRaised
        }
        border.width: control.visualFocus ? 2 : (control.tone === "primary" ? 0 : 1)
        border.color: control.visualFocus ? theme.accent : (control.tone === "danger" ? theme.dangerBorder : theme.border)

        Behavior on color { ColorAnimation { duration: 120 } }
    }
}
