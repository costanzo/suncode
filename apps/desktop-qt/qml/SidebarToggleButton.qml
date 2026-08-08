import QtQuick
import QtQuick.Controls

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

    contentItem: Canvas {
        id: iconCanvas
        anchors.fill: parent
        onPaint: {
            var ctx = getContext("2d")
            ctx.reset()
            ctx.strokeStyle = control.checked ? theme.text : theme.textSecondary
            ctx.fillStyle = ctx.strokeStyle
            ctx.lineWidth = 1.6
            ctx.lineCap = "round"
            ctx.lineJoin = "round"

            var w = width
            var h = height
            var barX = control.side === "right" ? w - 8 : 7
            ctx.fillRect(barX, 7, 3, h - 14)

            ctx.beginPath()
            if (control.side === "right") {
                ctx.moveTo(8, 9)
                ctx.lineTo(w - 12, 9)
                ctx.moveTo(8, h / 2)
                ctx.lineTo(w - 14, h / 2)
                ctx.moveTo(8, h - 9)
                ctx.lineTo(w - 16, h - 9)
            } else {
                ctx.moveTo(11, 9)
                ctx.lineTo(w - 8, 9)
                ctx.moveTo(14, h / 2)
                ctx.lineTo(w - 8, h / 2)
                ctx.moveTo(16, h - 9)
                ctx.lineTo(w - 8, h - 9)
            }
            ctx.stroke()
        }
    }
}
