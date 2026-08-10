import QtQuick
import QtQuick.Controls
import "../components"

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
        background: Rectangle {
            color: railButton.hovered ? theme.surfaceHover : "transparent"
        }
        contentItem: ThemeIcon {
            anchors.centerIn: parent
            width: 15
            height: 24
            source: root.side === "left"
                    ? "qrc:/assets/icons/sidebar-project.svg"
                    : "qrc:/assets/icons/sidebar-review.svg"
            color: railButton.hovered ? theme.accent : theme.textSecondary
        }
        onClicked: root.clicked()
    }
}
