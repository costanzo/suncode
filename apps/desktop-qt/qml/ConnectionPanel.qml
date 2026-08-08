import QtQuick
import QtQuick.Controls
import QtQuick.Effects
import QtQuick.Layouts

Rectangle {
    id: root
    property var client
    property var theme
    property bool collapsed: false
    property bool pinned: true
    signal collapseRequested()
    signal restoreRequested()
    signal pinToggled()

    // A RowLayout otherwise sizes this content panel to its implicit height (0).
    Layout.fillHeight: true

    function currentProjectName() {
        for (var index = 0; index < client.projects.length; index++) {
            var project = client.projects[index]
            if (project.projectId === client.projectId) {
                return project.displayName || project.canonicalRoot || "Project"
            }
        }
        return client.projectId.length > 0 ? "Opening project..." : "No project open"
    }

    color: theme.sidebar
    z: 2
    clip: true
    layer.enabled: true
    layer.effect: MultiEffect {
        shadowEnabled: true
        shadowColor: "#000000"
        shadowOpacity: 0.2
        shadowBlur: 0.3
        shadowHorizontalOffset: 3
        shadowVerticalOffset: 0
    }

    Rectangle { anchors.top: parent.top; anchors.bottom: parent.bottom; anchors.right: parent.right; width: 1; color: theme.border }

    SidebarRail {
        visible: root.collapsed
        anchors.fill: parent
        side: "left"
        label: "Open session sidebar"
        theme: root.theme
        onClicked: root.restoreRequested()
    }

    HoverHandler {
        id: panelHover
        enabled: !root.collapsed
        onHoveredChanged: {
            if (hovered) autoHideTimer.stop()
            else if (!root.pinned) autoHideTimer.restart()
        }
    }

    Timer {
        id: autoHideTimer
        interval: 420
        onTriggered: {
            if (!root.pinned && !panelHover.hovered && !root.collapsed) root.collapseRequested()
        }
    }

    ScrollView {
        visible: !root.collapsed
        anchors.fill: parent
        anchors.leftMargin: theme.panelPadding
        anchors.rightMargin: theme.panelPadding
        anchors.topMargin: 18
        anchors.bottomMargin: 14
        clip: true
        ScrollBar.horizontal.policy: ScrollBar.AlwaysOff

        ColumnLayout {
            width: root.width - theme.panelPadding * 2
            spacing: 10

            RowLayout {
                Layout.fillWidth: true
                Label { text: "PROJECT"; color: theme.textMuted; font.pixelSize: theme.typeCaption; font.weight: Font.DemiBold; font.letterSpacing: 1.1 }
                Item { Layout.fillWidth: true }
                Rectangle { width: 7; height: 7; radius: 4; color: client.connectionState === "connected" ? theme.success : theme.textMuted }
                SidebarLockButton { theme: root.theme; locked: root.pinned; onClicked: root.pinToggled() }
                SidebarToggleButton { theme: root.theme; checked: true; side: "left"; onClicked: root.collapseRequested() }
            }

            Text {
                Layout.fillWidth: true
                text: root.currentProjectName()
                color: theme.text
                font.pixelSize: theme.typeTitle
                font.weight: Font.DemiBold
                elide: Text.ElideMiddle
            }

            Text {
                Layout.fillWidth: true
                text: client.connectionState === "connected" ? "Local runtime connected" : client.statusText
                color: client.connectionState === "error" ? theme.danger : theme.textSecondary
                font.pixelSize: theme.typeLabel
                wrapMode: Text.Wrap
            }

            Rectangle { Layout.fillWidth: true; height: 1; color: theme.border; Layout.topMargin: 8; Layout.bottomMargin: 8 }

            SectionLabel { theme: root.theme; text: "SESSIONS" }

            Item {
                Layout.fillWidth: true
                implicitHeight: sessionColumn.implicitHeight

                Column {
                    id: sessionColumn
                    width: parent.width
                    spacing: 4

                    Repeater {
                        model: client.sessions
                        delegate: Rectangle {
                            required property var modelData
                            width: sessionColumn.width
                            height: 58
                            radius: theme.radiusSmall
                            color: modelData.sessionId === client.sessionId
                                ? theme.surfaceActive
                                : (modelData.status === "active" ? theme.surfaceHover : theme.surfaceRaised)
                            border.width: 1
                            border.color: modelData.sessionId === client.sessionId
                                ? theme.accentBorder
                                : (modelData.status === "active" ? theme.borderStrong : theme.border)

                            RowLayout {
                                anchors.fill: parent
                                anchors.margins: 10
                                spacing: 10

                                Rectangle {
                                    Layout.preferredWidth: 8
                                    Layout.preferredHeight: 8
                                    radius: 4
                                    color: modelData.status === "active" ? theme.success : theme.textMuted
                                }

                                ColumnLayout {
                                    Layout.fillWidth: true
                                    spacing: 2

                                    Label {
                                        Layout.fillWidth: true
                                        text: modelData.title || "Untitled session"
                                        color: theme.text
                                        font.pixelSize: theme.typeLabel
                                        font.weight: modelData.status === "active" ? Font.DemiBold : Font.Medium
                                        elide: Text.ElideRight
                                    }

                                    Label {
                                        Layout.fillWidth: true
                                        text: modelData.status === "active" ? "Active session" : "Archived session"
                                        color: modelData.status === "active" ? theme.success : theme.textMuted
                                        font.pixelSize: theme.typeCaption
                                        elide: Text.ElideRight
                                    }
                                }

                                Rectangle {
                                    visible: modelData.status === "active"
                                    Layout.preferredHeight: 20
                                    Layout.preferredWidth: activeLabel.implicitWidth + 12
                                    radius: 10
                                    color: theme.successSurface
                                    border.color: theme.accentBorder
                                    Text {
                                        id: activeLabel
                                        anchors.centerIn: parent
                                        text: "ACTIVE"
                                        color: theme.success
                                        font.pixelSize: theme.typeCaption
                                        font.weight: Font.DemiBold
                                    }
                                }
                            }

                            MouseArea {
                                anchors.fill: parent
                                cursorShape: Qt.PointingHandCursor
                                onClicked: client.selectSession(modelData.sessionId)
                            }
                        }
                    }
                }
            }

            Text { visible: client.sessions.length === 0; Layout.fillWidth: true; text: "No sessions yet"; color: theme.textMuted; font.pixelSize: theme.typeLabel }

            AppField { id: sessionTitle; Layout.fillWidth: true; theme: root.theme; placeholderText: "Session title"; onAccepted: client.renameSession(text) }

            AppButton {
                Layout.fillWidth: true
                theme: root.theme
                text: "New session"
                enabled: client.projectId.length > 0
                onClicked: client.createSession(sessionTitle.text)
            }

            RowLayout {
                Layout.fillWidth: true
                spacing: 8
                AppButton { Layout.fillWidth: true; theme: root.theme; compact: true; text: "Rename"; enabled: client.sessionId.length > 0 && sessionTitle.text.trim().length > 0; onClicked: client.renameSession(sessionTitle.text) }
                AppButton { Layout.fillWidth: true; theme: root.theme; compact: true; text: "Archive"; enabled: client.sessionId.length > 0; tone: "danger"; onClicked: client.archiveSession() }
            }

            Item { Layout.fillHeight: true; Layout.minimumHeight: 24 }
        }
    }
}
