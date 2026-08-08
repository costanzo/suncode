import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import Suncode.Runtime

ApplicationWindow {
    id: projectWindow
    property string projectIdToOpen: ""
    property var projectHub
    property bool projectBound: false
    property bool navigationVisible: true
    property bool processVisible: true
    property bool navigationPinned: true
    property bool processPinned: true
    property string pendingRestoreId: ""
    property var pendingRestorePaths: []
    readonly property var designTheme: theme

    visible: false
    width: 1440; height: 900; minimumWidth: 900; minimumHeight: 620
    title: "Suncode"
    color: theme.canvas

    Theme { id: theme }
    RuntimeClient { id: client; autoSelectProject: false }
    Binding { target: theme; property: "mode"; value: client.themeMode }

    palette.window: theme.canvas; palette.windowText: theme.text; palette.base: theme.field; palette.alternateBase: theme.surface; palette.text: theme.text; palette.button: theme.surfaceRaised; palette.buttonText: theme.text; palette.placeholderText: theme.textMuted; palette.highlight: theme.accent; palette.highlightedText: theme.accentInk; palette.toolTipBase: theme.surfaceRaised; palette.toolTipText: theme.text

    function openNewProject() { if (projectHub) projectHub.openNewProject(); else projectDialog.open() }
    function openSettings() { var component = Qt.createComponent("GlobalSettings.qml"); if (component.status === Component.Ready) { var settings = component.createObject(projectWindow, {}); if (settings) settings.show() } }
    function backToProjects() { projectWindow.close() }
    function currentProjectName() {
        for (var index = 0; index < client.projects.length; index++) {
            var project = client.projects[index]
            if (project.projectId === client.projectId) {
                return project.displayName || project.canonicalRoot || "Project"
            }
        }
        return client.projectId.length > 0 ? "Opening project..." : "No project open"
    }
    function maybeBindProject() {
        if (projectWindow.projectBound || projectWindow.projectIdToOpen.length === 0) {
            return
        }
        if (client.connectionState !== "connected") {
            return
        }
        projectWindow.projectBound = true
        client.selectProject(projectWindow.projectIdToOpen)
        projectWindow.raise()
        projectWindow.requestActivate()
    }

    onClosing: function(close) { if (projectHub) projectHub.projectWindowWillClose(projectWindow); close.accepted = true }
    Component.onCompleted: {
        client.connectToRuntime()
        maybeBindProject()
    }

    FolderDialog { id: projectDialog; title: "Open a local project"; onAccepted: client.openProject(selectedFolder) }
    Connections { target: client; function onProjectOpened(projectId) { projectWindow.title = "Suncode" } }
    Connections {
        target: client
        function onConnectionStateChanged() {
            projectWindow.maybeBindProject()
        }
    }

    menuBar: MenuBar {
        Menu {
            title: "Project"
            Action { text: "Open Project…"; onTriggered: projectWindow.openNewProject() }
            Action { text: "Back to Projects"; onTriggered: projectWindow.backToProjects() }
            MenuSeparator {}
            Action { text: "Close Window"; onTriggered: projectWindow.close() }
        }
        Menu { title: "Suncode"; Action { text: "Settings…"; onTriggered: projectWindow.openSettings() } }
    }

    Dialog {
        id: undoDialog; title: "Undo this turn's file changes?"; modal: true; anchors.centerIn: parent; width: Math.min(520, projectWindow.width - 48); standardButtons: Dialog.NoButton; closePolicy: Popup.CloseOnEscape
        background: Rectangle { color: theme.surfaceRaised; border.color: theme.borderStrong; radius: theme.radiusLarge }
        contentItem: ColumnLayout { spacing: 16
            Text { Layout.fillWidth: true; text: "Suncode will restore the files changed during this turn."; color: theme.text; font.pixelSize: theme.typeBody; wrapMode: Text.Wrap }
            Rectangle { Layout.fillWidth: true; implicitHeight: Math.min(150, restorePaths.implicitHeight + 24); color: theme.field; radius: theme.radiusMedium; border.color: theme.border; Text { id: restorePaths; anchors.fill: parent; anchors.margins: 12; text: projectWindow.pendingRestorePaths.join("\n"); color: theme.textSecondary; font.pixelSize: theme.typeLabel; wrapMode: Text.WrapAnywhere } }
            Text { Layout.fillWidth: true; text: "External side effects cannot be reversed."; color: theme.warning; font.pixelSize: theme.typeLabel; wrapMode: Text.Wrap }
            RowLayout {
                Layout.fillWidth: true
                Item { Layout.fillWidth: true }
                AppButton { theme: projectWindow.designTheme; text: "Cancel"; onClicked: undoDialog.close() }
                AppButton { theme: projectWindow.designTheme; text: "Undo changes"; tone: "primary"; onClicked: { client.restoreCheckpoint(projectWindow.pendingRestoreId); undoDialog.close() } }
            }
        }
    }

    RowLayout { anchors.fill: parent; spacing: 0
        ConnectionPanel {
            id: connectionPanel
            clip: true
            Layout.preferredWidth: projectWindow.navigationVisible ? Math.min(286, projectWindow.width * 0.24) : 24
            Layout.minimumWidth: 0
            Layout.maximumWidth: Math.min(300, projectWindow.width * 0.24)
            Behavior on Layout.preferredWidth { NumberAnimation { duration: 220; easing.type: Easing.OutCubic } }
            client: client
            theme: projectWindow.designTheme
            collapsed: !projectWindow.navigationVisible
            pinned: projectWindow.navigationPinned
            onCollapseRequested: projectWindow.navigationVisible = false
            onRestoreRequested: projectWindow.navigationVisible = true
            onPinToggled: projectWindow.navigationPinned = !projectWindow.navigationPinned
        }
        ConversationPanel { client: client; theme: projectWindow.designTheme; onSubmitRequested: function(text) { client.submitTurn(text) } }
        AgentProcessPanel {
            clip: true
            Layout.preferredWidth: projectWindow.processVisible ? Math.min(332, projectWindow.width * 0.27) : 24
            Layout.minimumWidth: 0
            Layout.maximumWidth: Math.min(352, projectWindow.width * 0.27)
            Behavior on Layout.preferredWidth { NumberAnimation { duration: 220; easing.type: Easing.OutCubic } }
            client: client
            theme: projectWindow.designTheme
            collapsed: !projectWindow.processVisible
            pinned: projectWindow.processPinned
            onCollapseRequested: projectWindow.processVisible = false
            onRestorePanelRequested: projectWindow.processVisible = true
            onPinToggled: projectWindow.processPinned = !projectWindow.processPinned
            onRestoreRequested: function(manifestId, paths) {
                projectWindow.pendingRestoreId = manifestId
                projectWindow.pendingRestorePaths = paths
                undoDialog.open()
            }
        }
    }
}
