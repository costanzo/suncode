import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts
import SunCode.Runtime
import "../shared/components"
import "../shared/theme"

ApplicationWindow {
    id: hub
    visible: true
    width: 980
    height: 680
    minimumWidth: 760
    minimumHeight: 520
    title: "Welcome to SunCode"
    color: theme.canvas

    property var projectWindows: []
    property var settingsWindow: null
    readonly property int projectRowInset: 14
    readonly property var designTheme: theme

    Theme { id: theme }
    RuntimeClient { id: hubClient; autoSelectProject: false }
    Binding { target: theme; property: "mode"; value: hubClient.themeMode }

    Action {
        text: "Settings…"
        enabled: hub.visible && hub.active
        shortcut: StandardKey.Preferences
        onTriggered: hub.openSettings(hub)
    }

    function openNewProject() { projectDialog.open() }

    function displayProjectPath(path) {
        var value = path || ""
        if (Qt.platform.os !== "windows") return value

        var uncPrefix = "\\\\?\\UNC\\"
        if (value.indexOf(uncPrefix) === 0) {
            return "\\\\" + value.substring(uncPrefix.length)
        }

        var localPrefix = "\\\\?\\"
        return value.indexOf(localPrefix) === 0
                ? value.substring(localPrefix.length)
                : value
    }

    function openProjectWindow(projectId) {
        if (!projectId || projectId.length === 0) {
            console.log("openProjectWindow called without projectId")
            return
        }
        for (var index = 0; index < projectWindows.length; index++) {
            var existingWindow = projectWindows[index]
            if (existingWindow && existingWindow.projectIdToOpen === projectId) {
                existingWindow.show()
                existingWindow.raise()
                existingWindow.requestActivate()
                return
            }
        }
        var component = Qt.createComponent("qrc:/qt/qml/SunCode/Desktop/qml/app/ProjectWindow.qml")
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

    function openSettings(ownerWindow) {
        var transientOwner = ownerWindow || hub
        if (settingsWindow) {
            settingsWindow.transientParent = transientOwner
            settingsWindow.show()
            settingsWindow.raise()
            settingsWindow.requestActivate()
            return
        }
        var component = Qt.createComponent("qrc:/qt/qml/SunCode/Desktop/qml/features/settings/GlobalSettings.qml")
        if (component.status !== Component.Ready) {
            console.log("GlobalSettings component not ready", component.errorString())
            return
        }
        settingsWindow = component.createObject(hub, { transientParent: transientOwner })
        if (settingsWindow) {
            settingsWindow.show()
            settingsWindow.raise()
            settingsWindow.requestActivate()
        }
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
            Image {
                Layout.preferredWidth: 30
                Layout.preferredHeight: 30
                source: "qrc:/assets/logo/suncode-logo-small-64.png"
                fillMode: Image.PreserveAspectFit
                smooth: true
                mipmap: true
            }
            Label { text: "SunCode"; color: theme.text; font.pixelSize: 19; font.weight: Font.DemiBold }
            Item { Layout.fillWidth: true }
            AppButton { theme: hub.designTheme; compact: true; text: "Settings"; onClicked: hub.openSettings(hub) }
            AppButton { theme: hub.designTheme; compact: true; text: "Open project"; tone: "primary"; onClicked: hub.openNewProject() }
        }
    }

    FolderDialog { id: projectDialog; title: "Open a local project"; onAccepted: hubClient.openProject(selectedFolder) }

    ColumnLayout {
        anchors.fill: parent; anchors.margins: 28; spacing: 18
        // ColumnLayout {
        //     Layout.fillWidth: true; spacing: 4
        //     Label { text: "Open a project"; color: theme.text; font.pixelSize: 27; font.weight: Font.DemiBold }
        //     Label { text: "Each project opens in its own window. Your sessions stay scoped to that project."; color: theme.textSecondary; font.pixelSize: theme.typeBody }
        // }

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
                }
                ListView {
                    Layout.fillWidth: true; Layout.fillHeight: true; clip: true; model: hubClient.projects; spacing: 8
                    delegate: Button {
                        id: projectRow
                        required property var modelData
                        width: ListView.view.width
                        height: 70
                        hoverEnabled: true
                        focusPolicy: Qt.TabFocus
                        padding: 0
                        Accessible.name: "Open project " + (modelData.displayName || hub.displayProjectPath(modelData.canonicalRoot))
                        onClicked: hub.openProjectWindow(modelData.projectId)

                        HoverHandler {
                            cursorShape: projectRow.enabled ? Qt.PointingHandCursor : Qt.ArrowCursor
                        }

                        background: Rectangle {
                            color: projectRow.hovered || projectRow.down ? theme.surfaceHover : theme.surfaceRaised
                            radius: theme.radiusMedium
                            border.width: projectRow.visualFocus ? 2 : 1
                            border.color: projectRow.visualFocus ? theme.accent : (projectRow.hovered ? theme.accentBorder : theme.border)
                        }

                        contentItem: Item {
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
                            Column {
                                anchors.left: projectBadge.right
                                anchors.leftMargin: 12
                                anchors.right: parent.right
                                anchors.rightMargin: hub.projectRowInset
                                anchors.verticalCenter: parent.verticalCenter
                                spacing: 3
                                Label { width: parent.width; text: projectRow.modelData.displayName; color: theme.text; font.pixelSize: theme.typeBody; font.weight: Font.DemiBold; elide: Text.ElideRight; horizontalAlignment: Text.AlignLeft }
                                Label { width: parent.width; text: hub.displayProjectPath(projectRow.modelData.canonicalRoot); color: theme.textMuted; font.family: theme.fontMono; font.pixelSize: theme.typeCaption; elide: Text.ElideMiddle; horizontalAlignment: Text.AlignLeft }
                            }
                        }
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
