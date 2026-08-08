import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import Suncode.Runtime

ApplicationWindow {
    id: hub
    visible: true
    width: 980
    height: 680
    minimumWidth: 760
    minimumHeight: 520
    title: "Suncode"
    color: theme.canvas

    property var projectWindows: []
    readonly property int projectRowInset: 14
    readonly property var designTheme: theme

    Theme { id: theme }
    RuntimeClient { id: hubClient; autoSelectProject: false }
    Binding { target: theme; property: "mode"; value: hubClient.themeMode }

    function openNewProject() { projectDialog.open() }

    function openProjectWindow(projectId) {
        if (!projectId || projectId.length === 0) {
            console.log("openProjectWindow called without projectId")
            return
        }
        var component = Qt.createComponent("ProjectWindow.qml")
        if (component.status !== Component.Ready) {
            console.log("ProjectWindow component not ready", component.errorString())
            return
        }
        var projectWindow = component.createObject(hub, { projectIdToOpen: projectId, projectHub: hub })
        if (!projectWindow) {
            console.log("ProjectWindow createObject returned null")
            return
        }
        projectWindows.push(projectWindow)
        projectWindow.show()
        projectWindow.raise()
        projectWindow.requestActivate()
        hub.hide()
    }

    function projectWindowWillClose(projectWindow) {
        var index = projectWindows.indexOf(projectWindow)
        if (index >= 0) projectWindows.splice(index, 1)
        if (projectWindows.length === 0) hub.show()
    }

    function openSettings() {
        var component = Qt.createComponent("GlobalSettings.qml")
        if (component.status !== Component.Ready) return
        var settingsWindow = component.createObject(hub, {})
        if (settingsWindow) settingsWindow.show()
    }

    Connections {
        target: hubClient
        function onProjectOpened(projectId) { hubClient.loadProjects(); hub.openProjectWindow(projectId) }
    }

    header: Rectangle {
        height: 62
        color: theme.surface
        Rectangle { anchors.left: parent.left; anchors.right: parent.right; anchors.bottom: parent.bottom; height: 1; color: theme.border }
        RowLayout {
            anchors.fill: parent; anchors.leftMargin: 22; anchors.rightMargin: 22; spacing: 12
            Label { text: "Suncode"; color: theme.text; font.pixelSize: 19; font.weight: Font.DemiBold }
            Item { Layout.fillWidth: true }
            AppButton { theme: hub.designTheme; compact: true; text: "Settings"; onClicked: hub.openSettings() }
            AppButton { theme: hub.designTheme; compact: true; text: "Open project"; tone: "primary"; onClicked: hub.openNewProject() }
        }
    }

    FolderDialog { id: projectDialog; title: "Open a local project"; onAccepted: hubClient.openProject(selectedFolder) }

    ColumnLayout {
        anchors.fill: parent; anchors.margins: 28; spacing: 18
        ColumnLayout {
            Layout.fillWidth: true; spacing: 4
            Label { text: "Open a project"; color: theme.text; font.pixelSize: 27; font.weight: Font.DemiBold }
            Label { text: "Each project opens in its own window. Your sessions stay scoped to that project."; color: theme.textSecondary; font.pixelSize: theme.typeBody }
        }

        Rectangle {
            Layout.fillWidth: true; Layout.fillHeight: true; color: theme.surface; radius: theme.radiusLarge; border.color: theme.border
            ColumnLayout {
                anchors.fill: parent; anchors.margins: 18; spacing: 12
                RowLayout {
                    Layout.fillWidth: true
                    Layout.leftMargin: hub.projectRowInset
                    Layout.rightMargin: hub.projectRowInset
                    SectionLabel { theme: hub.designTheme; text: "RECENT PROJECTS" }
                    Item { Layout.fillWidth: true }
                    Label { text: hubClient.projects.length + " projects"; color: theme.textMuted; font.pixelSize: theme.typeLabel }
                }
                ListView {
                    Layout.fillWidth: true; Layout.fillHeight: true; clip: true; model: hubClient.projects; spacing: 8
                    delegate: Rectangle {
                        required property var modelData
                        width: ListView.view.width; height: 70; color: projectMouse.containsMouse ? theme.surfaceHover : theme.surfaceRaised; radius: theme.radiusMedium; border.color: projectMouse.containsMouse ? theme.accentBorder : theme.border
                        Rectangle {
                            id: projectBadge
                            anchors.left: parent.left
                            anchors.leftMargin: hub.projectRowInset
                            anchors.verticalCenter: parent.verticalCenter
                            width: 34
                            height: 34
                            radius: 8
                            color: theme.accentSurface
                            Text { anchors.centerIn: parent; text: "P"; color: theme.accent; font.pixelSize: 15; font.weight: Font.DemiBold }
                        }
                        AppButton {
                            id: openProjectButton
                            anchors.right: parent.right
                            anchors.rightMargin: hub.projectRowInset
                            anchors.verticalCenter: parent.verticalCenter
                            theme: hub.designTheme
                            compact: true
                            text: "Open"
                            onClicked: { console.log("opening project", modelData.projectId); hub.openProjectWindow(modelData.projectId) }
                        }
                        Column {
                            anchors.left: projectBadge.right
                            anchors.leftMargin: 12
                            anchors.right: openProjectButton.left
                            anchors.rightMargin: 12
                            anchors.verticalCenter: parent.verticalCenter
                            spacing: 3
                            Label { width: parent.width; text: modelData.displayName; color: theme.text; font.pixelSize: theme.typeBody; font.weight: Font.DemiBold; elide: Text.ElideRight; horizontalAlignment: Text.AlignLeft }
                            Label { width: parent.width; text: modelData.canonicalRoot; color: theme.textMuted; font.pixelSize: theme.typeCaption; elide: Text.ElideMiddle; horizontalAlignment: Text.AlignLeft }
                        }
                        MouseArea { id: projectMouse; anchors.fill: parent; hoverEnabled: true; z: -1; acceptedButtons: Qt.NoButton }
                    }
                    footer: ColumnLayout {
                        width: parent.width; spacing: 10; visible: hubClient.projects.length === 0
                        Label { Layout.fillWidth: true; topPadding: 48; text: "No projects yet"; color: theme.text; font.pixelSize: theme.typeHeading; font.weight: Font.DemiBold; horizontalAlignment: Text.AlignHCenter }
                        Label { Layout.fillWidth: true; text: "Open a local folder to create your first project window."; color: theme.textSecondary; font.pixelSize: theme.typeBody; horizontalAlignment: Text.AlignHCenter }
                    }
                }
            }
        }
    }
}
