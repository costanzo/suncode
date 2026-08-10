import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import "../features/conversation"
import "../features/project"
import "../features/review"
import "../shared/components"
import "../shared/navigation"
import "../shared/theme"

ApplicationWindow {
    id: window

    /*
      THESIS: Suncode is a quiet control desk; the conversation is the work surface, not one equal panel in a dashboard.
      OWN-WORLD: Matte graphite fields, hairline separators, restrained teal interaction, and amber authority states.
      STORY: Choose a local project, direct the agent, review sensitive actions, and undo filesystem changes without losing focus.
      FIRST VIEWPORT: A wide conversation canvas between independently collapsible navigation and review tool bays; panel controls live in the top bar.
      FORM: Professional editing-console grammar, assigned direction seed 85a6895a.
      FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, and DESIGN.md
    */

    visible: true
    width: 1440
    height: 900
    minimumWidth: 900
    minimumHeight: 620
    title: "Suncode"
    color: theme.canvas

    property bool navigationVisible: true
    property bool reviewVisible: true
    property string pendingRestoreId: ""
    property var pendingRestorePaths: []
    readonly property var designTheme: theme

    Theme { id: theme }
    Binding { target: theme; property: "mode"; value: runtimeClient.themeMode }

    palette.window: theme.canvas
    palette.windowText: theme.text
    palette.base: theme.field
    palette.alternateBase: theme.surface
    palette.text: theme.text
    palette.button: theme.surfaceRaised
    palette.buttonText: theme.text
    palette.placeholderText: theme.textMuted
    palette.highlight: theme.accent
    palette.highlightedText: theme.accentInk
    palette.toolTipBase: theme.surfaceRaised
    palette.toolTipText: theme.text
    palette.brightText: theme.danger

    FolderDialog {
        id: projectFolderDialog
        title: "Open a local project"
        onAccepted: runtimeClient.openProject(selectedFolder)
    }

    Dialog {
        id: undoDialog
        title: "Undo this turn's file changes?"
        modal: true
        anchors.centerIn: parent
        width: Math.min(520, window.width - 48)
        standardButtons: Dialog.NoButton
        closePolicy: Popup.CloseOnEscape

        background: Rectangle {
            color: theme.surfaceRaised
            border.color: theme.borderStrong
            radius: theme.radiusLarge
        }

        contentItem: ColumnLayout {
            spacing: 16

            Text {
                Layout.fillWidth: true
                text: "Suncode will restore the files changed during this turn."
                color: theme.text
                font.pixelSize: theme.typeBody
                wrapMode: Text.Wrap
            }

            Rectangle {
                Layout.fillWidth: true
                implicitHeight: Math.min(150, restorePaths.implicitHeight + 24)
                color: theme.field
                radius: theme.radiusMedium
                border.color: theme.border

                Text {
                    id: restorePaths
                    anchors.fill: parent
                    anchors.margins: 12
                    text: window.pendingRestorePaths.join("\n")
                    color: theme.textSecondary
                    font.family: theme.fontMono
                    font.pixelSize: theme.typeLabel
                    wrapMode: Text.WrapAnywhere
                    elide: Text.ElideRight
                }
            }

            Text {
                Layout.fillWidth: true
                text: "External side effects—such as pushed commits, published packages, or network requests—cannot be reversed."
                color: theme.warning
                font.pixelSize: theme.typeLabel
                wrapMode: Text.Wrap
            }

            RowLayout {
                Layout.fillWidth: true
                Item { Layout.fillWidth: true }
                AppButton { theme: window.designTheme; text: "Cancel"; onClicked: undoDialog.close() }
                AppButton {
                    theme: window.designTheme
                    text: "Undo changes"
                    tone: "primary"
                    onClicked: {
                        runtimeClient.restoreCheckpoint(window.pendingRestoreId)
                        undoDialog.close()
                    }
                }
            }
        }
    }

    header: Rectangle {
        height: 54
        color: theme.surface

        Rectangle {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            height: 1
            color: theme.border
        }

        RowLayout {
            anchors.fill: parent
            anchors.leftMargin: 12
            anchors.rightMargin: 16
            spacing: 10

            SidebarToggleButton {
                theme: window.designTheme
                checked: window.navigationVisible
                side: "left"
                Accessible.name: window.navigationVisible ? "Hide project navigation" : "Show project navigation"
                onClicked: window.navigationVisible = !window.navigationVisible
            }

            Label {
                text: "Suncode"
                color: theme.text
                font.pixelSize: theme.typeTitle
                font.weight: Font.DemiBold
                leftPadding: 4
            }

            Rectangle { Layout.preferredWidth: 1; Layout.preferredHeight: 20; color: theme.border }

            Label {
                Layout.fillWidth: true
                text: runtimeClient.sessionTitle.length > 0 ? runtimeClient.sessionTitle : (runtimeClient.sessionId.length > 0 ? runtimeClient.sessionId : "No session selected")
                color: runtimeClient.sessionId.length > 0 ? theme.textSecondary : theme.textMuted
                font.pixelSize: theme.typeLabel
                font.weight: Font.Medium
                elide: Text.ElideMiddle
                horizontalAlignment: Text.AlignHCenter
            }

            RowLayout {
                spacing: 8
                Rectangle {
                    width: 8
                    height: 8
                    radius: 4
                    color: runtimeClient.connectionState === "connected" ? theme.success
                         : runtimeClient.connectionState === "connecting" || runtimeClient.connectionState === "reconnecting" ? theme.warning
                         : runtimeClient.connectionState === "error" ? theme.danger : theme.textMuted
                }
                Label {
                    text: runtimeClient.statusText
                    color: runtimeClient.connectionState === "error" ? theme.danger : theme.textSecondary
                    font.pixelSize: theme.typeLabel
                    elide: Text.ElideRight
                    Layout.maximumWidth: Math.max(120, window.width * 0.22)
                }
            }

            SidebarToggleButton {
                theme: window.designTheme
                checked: window.reviewVisible
                side: "right"
                Accessible.name: window.reviewVisible ? "Hide review panel" : "Show review panel"
                onClicked: window.reviewVisible = !window.reviewVisible
            }
        }
    }

    RowLayout {
        anchors.fill: parent
        spacing: 0

        ConnectionPanel {
            id: connectionPanel
            clip: true
            Layout.preferredWidth: window.navigationVisible ? Math.min(286, window.width * 0.24) : 0
            Layout.minimumWidth: 0
            Layout.maximumWidth: Math.min(300, window.width * 0.24)
            client: runtimeClient
                theme: window.designTheme
        }

        ConversationPanel {
            client: runtimeClient
            theme: window.designTheme
            onSubmitRequested: function(text) { runtimeClient.submitTurn(text) }
        }

        ReviewPanel {
            clip: true
            Layout.preferredWidth: window.reviewVisible ? Math.min(332, window.width * 0.27) : 0
            Layout.minimumWidth: 0
            Layout.maximumWidth: Math.min(352, window.width * 0.27)
            client: runtimeClient
            theme: window.designTheme
            onRestoreRequested: function(manifestId, paths) {
                window.pendingRestoreId = manifestId
                window.pendingRestorePaths = paths
                undoDialog.open()
            }
        }
    }
}
