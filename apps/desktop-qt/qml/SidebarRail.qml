import QtQuick
import QtQuick.Controls

Rectangle {
    id: root
    property var theme
    property string side: "left"
    property string label: "Open sidebar"
    signal clicked()

    color: theme.sidebar
    border.color: theme.border
    border.width: 1
    z: 10

    Button {
        id: railButton
        anchors.fill: parent
        hoverEnabled: true
        focusPolicy: Qt.TabFocus
        Accessible.name: root.label
        ToolTip.visible: hovered
        ToolTip.text: root.label
        background: Rectangle {
            color: railButton.hovered ? theme.surfaceHover : "transparent"
        }
        contentItem: Canvas {
            anchors.centerIn: parent
            width: 12
            height: 34
            onPaint: {
                var ctx = getContext("2d")
                ctx.reset()
                ctx.strokeStyle = railButton.hovered ? theme.accent : theme.textSecondary
                ctx.fillStyle = ctx.strokeStyle
                ctx.lineWidth = 1.4
                ctx.lineCap = "round"
                ctx.lineJoin = "round"
                if (root.side === "left") {
                    ctx.strokeRect(2, 10, 8, 5)
                    ctx.strokeRect(2, 18, 8, 5)
                    ctx.fillRect(3, 12, 2, 1.5)
                    ctx.fillRect(3, 20, 2, 1.5)
                } else {
                    ctx.beginPath()
                    ctx.moveTo(3, 12)
                    ctx.lineTo(9, 12)
                    ctx.moveTo(3, 17)
                    ctx.lineTo(9, 17)
                    ctx.moveTo(3, 22)
                    ctx.lineTo(9, 22)
                    ctx.stroke()
                    ctx.beginPath()
                    ctx.arc(2, 12, 1, 0, Math.PI * 2)
                    ctx.arc(2, 17, 1, 0, Math.PI * 2)
                    ctx.arc(2, 22, 1, 0, Math.PI * 2)
                    ctx.fill()
                }
            }
        }
        onClicked: root.clicked()
    }
}
