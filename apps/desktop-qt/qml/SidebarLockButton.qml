import QtQuick
import QtQuick.Controls

Button {
    id: control
    property var theme
    property bool locked: true

    implicitWidth: 26
    implicitHeight: 26
    hoverEnabled: true
    focusPolicy: Qt.TabFocus
    Accessible.name: locked ? "Unlock sidebar" : "Lock sidebar open"
    ToolTip.visible: hovered
    ToolTip.text: locked ? "Unlock sidebar" : "Lock sidebar open"

    background: Rectangle {
        radius: theme.radiusSmall
        color: control.hovered ? theme.surfaceHover : "transparent"
        border.width: control.visualFocus ? 2 : 0
        border.color: theme.accent
    }

    contentItem: Canvas {
        anchors.centerIn: parent
        width: 16
        height: 16
        onPaint: {
            var ctx = getContext("2d")
            ctx.reset()
            ctx.strokeStyle = control.locked ? theme.accent : theme.textMuted
            ctx.lineWidth = 1.4
            ctx.lineCap = "round"
            ctx.lineJoin = "round"
            ctx.beginPath()
            if (control.locked) {
                ctx.moveTo(5, 7)
                ctx.lineTo(5, 5)
                ctx.arc(8, 5, 3, Math.PI, 0)
                ctx.lineTo(11, 7)
            } else {
                ctx.moveTo(5, 7)
                ctx.lineTo(5, 5)
                ctx.arc(8, 5, 3, Math.PI, 0.25)
                ctx.lineTo(11, 7)
            }
            ctx.stroke()
            ctx.strokeRect(3, 7, 10, 7)
            ctx.fillStyle = ctx.strokeStyle
            ctx.fillRect(7.2, 10, 1.6, 2.5)
        }
    }
}
