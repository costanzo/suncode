import QtQuick
import QtQuick.Controls
import QtQuick.Effects
import QtQuick.Layouts
import "../../shared/components"
import "../../shared/navigation"

Rectangle {
    id: root
    property var client
    property var theme
    property bool collapsed: false
    property bool pinned: true
    property bool cardMode: false
    signal collapseRequested()
    signal restoreRequested()
    signal pinToggled()

    // A RowLayout otherwise sizes this content panel to its implicit height (0).
    Layout.fillHeight: true

    function openCreateSessionDialog() {
        sessionDialog.mode = "create"
        sessionDialog.sessionId = ""
        sessionDialog.titleText = ""
        sessionDialog.open()
    }

    function openRenameSessionDialog(sessionId, title) {
        sessionDialog.mode = "rename"
        sessionDialog.sessionId = sessionId
        sessionDialog.titleText = title || "Untitled session"
        sessionDialog.open()
    }

    function sessionRelativeTime(value) {
        if (!value || value.length === 0) {
            return "No activity yet"
        }
        var then = new Date(value)
        if (isNaN(then.getTime())) {
            return value
        }
        var seconds = Math.max(0, Math.floor((Date.now() - then.getTime()) / 1000))
        if (seconds < 60) return "Just now"
        var minutes = Math.floor(seconds / 60)
        if (minutes < 60) return minutes + "m ago"
        var hours = Math.floor(minutes / 60)
        if (hours < 24) return hours + "h ago"
        var days = Math.floor(hours / 24)
        if (days < 7) return days + "d ago"
        return then.toLocaleDateString(Qt.locale(), Locale.ShortFormat)
    }

    color: cardMode ? "transparent" : theme.sidebar
    z: 2
    clip: true
    layer.enabled: true
    layer.effect: MultiEffect {
        shadowEnabled: !root.cardMode
        shadowColor: "#000000"
        shadowOpacity: 0.2
        shadowBlur: 0.3
        shadowHorizontalOffset: 3
        shadowVerticalOffset: 0
    }

    Rectangle { visible: !root.cardMode; anchors.top: parent.top; anchors.bottom: parent.bottom; anchors.right: parent.right; width: 1; color: theme.border }

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

    Dialog {
        id: sessionDialog
        property string mode: "create"
        property string sessionId: ""
        property string titleText: ""

        modal: true
        anchors.centerIn: Overlay.overlay
        width: Math.min(360, root.Window.window ? root.Window.window.width - 56 : 360)
        title: mode === "create" ? "New session" : "Rename session"
        standardButtons: Dialog.NoButton
        closePolicy: Popup.CloseOnEscape | Popup.CloseOnPressOutside
        leftPadding: 20
        rightPadding: 20
        topPadding: 8
        bottomPadding: 18

        onOpened: {
            titleInput.text = titleText
            titleInput.selectAll()
            titleInput.forceActiveFocus()
        }

        function submit() {
            var value = titleInput.text.trim()
            if (value.length === 0) {
                return
            }
            if (mode === "create") {
                client.createSession(value)
            } else {
                client.renameSessionById(sessionId, value)
            }
            close()
        }

        background: Rectangle {
            color: theme.surfaceRaised
            border.color: theme.borderStrong
            radius: theme.radiusLarge
        }

        header: Label {
            text: sessionDialog.title
            color: theme.text
            font.pixelSize: theme.typeTitle
            font.weight: Font.DemiBold
            horizontalAlignment: Text.AlignHCenter
            leftPadding: 20
            rightPadding: 20
            topPadding: 18
            bottomPadding: 8
        }

        contentItem: ColumnLayout {
            spacing: 14

            AppField {
                id: titleInput
                Layout.fillWidth: true
                theme: root.theme
                placeholderText: "Session name"
                onAccepted: sessionDialog.submit()
            }

            RowLayout {
                Layout.fillWidth: true
                Item { Layout.fillWidth: true }
                AppButton {
                    theme: root.theme
                    text: "Cancel"
                    compact: true
                    onClicked: sessionDialog.close()
                }
                AppButton {
                    theme: root.theme
                    text: sessionDialog.mode === "create" ? "Create" : "Save"
                    tone: "primary"
                    compact: true
                    enabled: titleInput.text.trim().length > 0
                    onClicked: sessionDialog.submit()
                }
            }
        }
    }

    ColumnLayout {
        visible: !root.collapsed
        anchors.fill: parent
        anchors.leftMargin: theme.panelPadding
        anchors.rightMargin: theme.panelPadding
        anchors.topMargin: 14
        anchors.bottomMargin: 12
        spacing: 10

        RowLayout {
            Layout.fillWidth: true
            Layout.preferredHeight: 34
            spacing: 8

            SectionLabel {
                Layout.fillWidth: true
                theme: root.theme
                text: "SESSIONS"
            }

            Button {
                id: newSessionButton
                Layout.preferredWidth: 30
                Layout.preferredHeight: 30
                enabled: client.projectId.length > 0
                hoverEnabled: true
                Accessible.name: "Create session"
                ToolTip.visible: hovered
                ToolTip.text: "New session"
                onClicked: root.openCreateSessionDialog()

                background: Rectangle {
                    radius: theme.radiusSmall
                    color: newSessionButton.down
                        ? theme.surfaceActive
                        : (newSessionButton.hovered ? theme.surfaceHover : "transparent")
                    border.width: newSessionButton.visualFocus ? 2 : 1
                    border.color: newSessionButton.visualFocus
                        ? theme.accent
                        : (newSessionButton.hovered ? theme.borderStrong : theme.border)
                }

                contentItem: ThemeIcon {
                    anchors.centerIn: parent
                    width: 15
                    height: 15
                    source: "qrc:/assets/icons/plus.svg"
                    color: newSessionButton.enabled ? theme.textSecondary : theme.textDisabled
                    opacity: newSessionButton.enabled ? 1 : 0.45
                }
            }
        }

        ListView {
            id: sessionList
            Layout.fillWidth: true
            Layout.fillHeight: true
            clip: true
            model: client.sessions
            spacing: 4
            boundsBehavior: Flickable.StopAtBounds
            ScrollBar.vertical: ScrollBar {
                policy: ScrollBar.AsNeeded
            }

            delegate: Button {
                id: sessionItem
                required property var modelData
                readonly property bool selected: modelData.sessionId === client.sessionId
                readonly property string sessionTitle: modelData.title || "Untitled session"

                width: ListView.view.width
                height: 48
                hoverEnabled: true
                focusPolicy: Qt.TabFocus
                padding: 0
                Accessible.name: "Open session " + sessionTitle
                onClicked: client.selectSession(modelData.sessionId)

                background: Rectangle {
                    color: sessionItem.selected
                        ? theme.surfaceActive
                        : (sessionItem.hovered ? theme.surfaceHover : "transparent")
                    radius: theme.radiusSmall
                    border.width: sessionItem.visualFocus || sessionItem.selected ? 1 : 0
                    border.color: sessionItem.visualFocus
                        ? theme.accent
                        : (sessionItem.selected ? theme.borderStrong : "transparent")
                }

                contentItem: Item {
                    anchors.fill: parent

                    Column {
                        anchors.left: parent.left
                        anchors.leftMargin: 10
                        anchors.right: sessionMenuButton.left
                        anchors.rightMargin: 8
                        anchors.verticalCenter: parent.verticalCenter
                        spacing: 2

                        Label {
                            width: parent.width
                            text: sessionItem.sessionTitle
                            color: sessionItem.selected ? theme.text : theme.textSecondary
                            font.pixelSize: theme.typeLabel
                            font.weight: sessionItem.selected ? Font.DemiBold : Font.Medium
                            elide: Text.ElideRight
                        }

                        Label {
                            width: parent.width
                            text: root.sessionRelativeTime(modelData.lastActivityAt)
                            color: theme.textMuted
                            font.pixelSize: theme.typeCaption
                            elide: Text.ElideRight
                        }
                    }

                    Button {
                        id: sessionMenuButton
                        anchors.right: parent.right
                        anchors.rightMargin: 4
                        anchors.verticalCenter: parent.verticalCenter
                        width: 28
                        height: 28
                        padding: 0
                        hoverEnabled: true
                        Accessible.name: "Session actions"
                        ToolTip.visible: hovered
                        ToolTip.text: "Session actions"
                        onClicked: sessionActions.open()

                        background: Rectangle {
                            radius: theme.radiusSmall
                            color: sessionMenuButton.down
                                ? theme.surfaceActive
                                : (sessionMenuButton.hovered || sessionActions.opened ? theme.surfaceActive : "transparent")
                            border.width: sessionMenuButton.visualFocus ? 2 : 0
                            border.color: theme.accent
                        }

                        contentItem: ThemeIcon {
                            anchors.centerIn: parent
                            width: 16
                            height: 16
                            source: "qrc:/assets/icons/more-horizontal.svg"
                            color: sessionMenuButton.hovered || sessionActions.opened
                                   ? theme.textSecondary
                                   : theme.textMuted
                        }

                        Menu {
                            id: sessionActions
                            y: sessionMenuButton.height
                            implicitWidth: 132
                            popupType: Popup.Item
                            padding: 4
                            margins: 8

                            background: Rectangle {
                                color: theme.surfaceRaised
                                radius: theme.radiusMedium
                                border.width: 1
                                border.color: theme.borderStrong
                            }

                            MenuItem {
                                id: renameSessionAction
                                text: "Rename"
                                implicitHeight: 34
                                hoverEnabled: true

                                contentItem: Label {
                                    leftPadding: 10
                                    rightPadding: 10
                                    text: renameSessionAction.text
                                    color: theme.textSecondary
                                    font.pixelSize: theme.typeLabel
                                    verticalAlignment: Text.AlignVCenter
                                }

                                background: Rectangle {
                                    radius: theme.radiusSmall
                                    color: renameSessionAction.down
                                        ? theme.surfaceActive
                                        : (renameSessionAction.highlighted || renameSessionAction.hovered
                                           ? theme.surfaceHover
                                           : "transparent")
                                    border.width: renameSessionAction.visualFocus ? 2 : 0
                                    border.color: theme.accent
                                }

                                onTriggered: root.openRenameSessionDialog(modelData.sessionId, sessionItem.sessionTitle)
                            }
                            MenuItem {
                                id: archiveSessionAction
                                text: "Archive"
                                implicitHeight: 34
                                hoverEnabled: true

                                contentItem: Label {
                                    leftPadding: 10
                                    rightPadding: 10
                                    text: archiveSessionAction.text
                                    color: theme.textSecondary
                                    font.pixelSize: theme.typeLabel
                                    verticalAlignment: Text.AlignVCenter
                                }

                                background: Rectangle {
                                    radius: theme.radiusSmall
                                    color: archiveSessionAction.down
                                        ? theme.surfaceActive
                                        : (archiveSessionAction.highlighted || archiveSessionAction.hovered
                                           ? theme.surfaceHover
                                           : "transparent")
                                    border.width: archiveSessionAction.visualFocus ? 2 : 0
                                    border.color: theme.accent
                                }

                                onTriggered: client.archiveSessionById(modelData.sessionId)
                            }
                        }
                    }
                }
            }

            footer: ColumnLayout {
                width: sessionList.width
                visible: client.sessions.length === 0
                spacing: 6

                Label {
                    Layout.fillWidth: true
                    topPadding: 36
                    text: "No sessions yet"
                    color: theme.textSecondary
                    font.pixelSize: theme.typeBody
                    font.weight: Font.DemiBold
                    horizontalAlignment: Text.AlignHCenter
                }

                Label {
                    Layout.fillWidth: true
                    text: "Use + to create one."
                    color: theme.textMuted
                    font.pixelSize: theme.typeLabel
                    horizontalAlignment: Text.AlignHCenter
                }
            }
        }
    }
}
