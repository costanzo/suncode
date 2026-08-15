import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Effects
import QtQuick.Layouts
import QtQuick.Window
import SunCode.Runtime
import "../features/conversation"
import "../features/git"
import "../features/project"
import "../features/review"
import "../features/settings"
import "../shared/components"
import "../shared/navigation"
import "../shared/theme"
import "../shared/window"

ApplicationWindow {
    id: projectWindow
    property string projectIdToOpen: ""
    property var projectHub
    property bool projectBound: false
    property bool navigationVisible: true
    property bool processVisible: true
    property bool gitDrawerVisible: false
    property real gitDrawerHeight: 340
    property bool navigationPinned: true
    property string pendingRestoreId: ""
    property var pendingRestorePaths: []
    property var settingsWindow: null
    readonly property var designTheme: theme
    readonly property bool isMacOS: Qt.platform.os === "osx"
    readonly property bool isFullScreen: projectWindow.visibility === Window.FullScreen
    readonly property bool isMaximized: projectWindow.visibility === Window.Maximized
    readonly property bool windowShadowVisible: !isMacOS && !isFullScreen && !isMaximized
    readonly property int windowShadowInset: windowShadowVisible ? 10 : 0
    readonly property int titleBarHeight: 36
    readonly property int footerHeight: 24
    readonly property bool roundedWindowChrome: isMacOS && !projectWindow.isFullScreen && projectWindow.visibility !== Window.Maximized
    readonly property int windowCornerRadius: roundedWindowChrome ? 12 : 0
    readonly property int resizeHandleSize: isMacOS ? 5 : 6
    readonly property int chromeHorizontalInset: isFullScreen ? 0 : 4
    readonly property int chromeVerticalInset: isFullScreen ? 0 : (isMacOS ? 6 : 4)
    readonly property int chromeGap: 4
    readonly property int conversationContentMaximumWidth: 780
    readonly property int windowsCaptionGlyphSize: 12
    readonly property color windowsCaptionForeground: theme.isLight ? "#000000" : "#ffffff"
    readonly property color windowsCloseHover: "#E81123"
    readonly property color windowsClosePressed: "#a82318"

    function compactTokenCount(value) {
        if (value < 1000) {
            return Number(value).toLocaleString(Qt.locale(), "f", 0)
        }
        var divisor = value < 1000000 ? 1000 : 1000000
        var suffix = value < 1000000 ? "k" : "m"
        var scaled = value / divisor
        var precision = scaled < 100 ? 1 : 0
        return Number(scaled).toLocaleString(Qt.locale(), "f", precision) + suffix
    }

    visible: false
    width: 1440; height: 900; minimumWidth: 900; minimumHeight: 620
    flags: Qt.Window | Qt.FramelessWindowHint
    transientParent: null
    title: currentProjectName()
    color: "transparent"

    Theme { id: theme }
    RuntimeClient { id: client; autoSelectProject: false }
    WindowStateController { id: windowState; window: projectWindow }
    Binding { target: theme; property: "mode"; value: client.themeMode }

    palette.window: theme.canvas; palette.windowText: theme.text; palette.base: theme.field; palette.alternateBase: theme.surface; palette.text: theme.text; palette.button: theme.surfaceRaised; palette.buttonText: theme.text; palette.placeholderText: theme.textMuted; palette.highlight: theme.accent; palette.highlightedText: theme.accentInk; palette.toolTipBase: theme.surfaceRaised; palette.toolTipText: theme.text

    menuBar: MenuBar {
        visible: projectWindow.isMacOS
        Menu {
            title: "Project actions"
            Action { text: "Open Project…"; onTriggered: projectWindow.openNewProject() }
            Action { text: "Back to Projects"; onTriggered: projectWindow.backToProjects() }
            Menu {
                id: recentProjectsMenu
                title: "Open Recent Project"
                enabled: client.projects.length > 0
                Instantiator {
                    model: client.projects
                    delegate: Action {
                        required property var modelData
                        text: modelData.displayName || modelData.canonicalRoot
                        enabled: !projectWindow.recentProjectIsOpen(modelData.projectId)
                        onTriggered: projectWindow.focusRecentProject(modelData.projectId)
                    }
                    onObjectAdded: function(index, object) { recentProjectsMenu.addAction(object) }
                    onObjectRemoved: function(index, object) { recentProjectsMenu.removeAction(object) }
                }
            }
            Action { text: toggleNavigationAction.text; enabled: projectWindow.visible; onTriggered: projectWindow.toggleNavigation() }
            MenuSeparator {}
            Action { text: "Settings…"; enabled: projectWindow.visible && projectWindow.active; shortcut: StandardKey.Preferences; onTriggered: projectWindow.openSettings() }
            Action { text: "Close Window"; onTriggered: projectWindow.close() }
        }
    }

    function openNewProject() { if (projectHub) projectHub.openNewProject(); else projectDialog.open() }
    function openSettings() {
        if (projectHub) {
            projectHub.openSettings(projectWindow)
            return
        }
        if (settingsWindow) {
            settingsWindow.transientParent = projectWindow
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
        settingsWindow = component.createObject(projectWindow, { transientParent: projectWindow })
        if (settingsWindow) {
            settingsWindow.show()
            settingsWindow.raise()
            settingsWindow.requestActivate()
        }
    }
    function backToProjects() { projectWindow.close() }
    function openRecentProject(projectId) {
        if (!projectId || projectId.length === 0) {
            return
        }
        if (projectHub) {
            projectHub.openProjectWindow(projectId)
            return
        }
        var component = Qt.createComponent("qrc:/qt/qml/SunCode/Desktop/qml/app/ProjectWindow.qml")
        if (component.status !== Component.Ready) {
            console.log("ProjectWindow component not ready", component.errorString())
            return
        }
        var recentProjectWindow = component.createObject(projectWindow, { projectIdToOpen: projectId, projectHub: projectWindow.projectHub })
        if (!recentProjectWindow) {
            console.log("ProjectWindow createObject returned null for recent project")
            return
        }
        recentProjectWindow.show()
        recentProjectWindow.raise()
        recentProjectWindow.requestActivate()
    }
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
    Connections {
        target: client
        function onConnectionStateChanged() {
            projectWindow.maybeBindProject()
        }
    }

    function toggleMaximized() {
        if (projectWindow.isMacOS) {
            windowState.toggleFullScreen()
        } else {
            windowState.toggleMaximized()
        }
    }

    function toggleNavigation() {
        projectWindow.navigationVisible = !projectWindow.navigationVisible
    }

    function toggleGitDrawer() {
        projectWindow.gitDrawerVisible = !projectWindow.gitDrawerVisible
        if (projectWindow.gitDrawerVisible) {
            client.refreshGitStatus()
        }
    }

    function recentProjectIsOpen(projectId) {
        if (!projectHub || !projectId) {
            return false
        }
        for (var index = 0; index < projectHub.projectWindows.length; index++) {
            var existingWindow = projectHub.projectWindows[index]
            if (existingWindow && existingWindow.projectIdToOpen === projectId) {
                return true
            }
        }
        return false
    }

    function focusRecentProject(projectId) {
        if (!projectHub || !projectId) {
            return
        }
        for (var index = 0; index < projectHub.projectWindows.length; index++) {
            var existingWindow = projectHub.projectWindows[index]
            if (existingWindow && existingWindow.projectIdToOpen === projectId) {
                existingWindow.show()
                existingWindow.raise()
                existingWindow.requestActivate()
                return
            }
        }
        projectWindow.openRecentProject(projectId)
    }

    Action {
        id: toggleNavigationAction
        text: projectWindow.navigationVisible ? "Hide Project Navigation" : "Show Project Navigation"
        enabled: projectWindow.visible && projectWindow.active
        shortcut: "Ctrl+1"
        onTriggered: projectWindow.toggleNavigation()
    }

    component TitleBarButton: Button {
        id: control
        property var theme
        property string kind: "menu"
        property bool danger: false
        property bool macStyle: false
        property bool windowsCaption: false

        hoverEnabled: true
        focusPolicy: Qt.TabFocus
        implicitWidth: control.macStyle ? 18 : (control.windowsCaption ? 46 : (projectWindow.isMacOS ? 24 : 32))
        implicitHeight: control.macStyle ? 18 : (control.windowsCaption || !projectWindow.isMacOS ? projectWindow.titleBarHeight : 24)
        padding: 0

        background: Rectangle {
            visible: !control.macStyle
            radius: control.windowsCaption ? 0 : (projectWindow.isMacOS ? width / 2 : theme.radiusSmall)
            color: {
                if (control.windowsCaption && control.kind === "close") {
                    if (control.down) return projectWindow.windowsClosePressed
                    if (control.hovered) return projectWindow.windowsCloseHover
                    return "transparent"
                }
                return control.down ? theme.surfaceActive : (control.hovered ? theme.surfaceHover : "transparent")
            }
            border.width: control.visualFocus ? 2 : 0
            border.color: control.visualFocus ? theme.accent : "transparent"
        }

        contentItem: Item {
            implicitWidth: control.implicitWidth
            implicitHeight: control.implicitHeight

            ThemeIcon {
                visible: !control.macStyle
                anchors.centerIn: parent
                width: control.kind === "gear" ? 16 : (control.windowsCaption ? projectWindow.windowsCaptionGlyphSize : 16)
                height: width
                source: {
                    if (control.kind === "gear") return "qrc:/assets/icons/settings.svg"
                    if (control.windowsCaption && control.kind === "minus") return "qrc:/assets/icons/windows-minimize.svg"
                    if (control.windowsCaption && control.kind === "maximize") return "qrc:/assets/icons/windows-maximize.svg"
                    if (control.windowsCaption && control.kind === "close") return "qrc:/assets/icons/windows-close.svg"
                    if (control.kind === "minus") return "qrc:/assets/icons/minus.svg"
                    if (control.kind === "restore") return "qrc:/assets/icons/restore.svg"
                    if (control.kind === "maximize") return "qrc:/assets/icons/maximize.svg"
                    if (control.kind === "close") return "qrc:/assets/icons/close.svg"
                    return "qrc:/assets/icons/more-horizontal.svg"
                }
                color: !control.enabled
                       ? theme.textDisabled
                       : control.windowsCaption && control.kind === "close" && (control.hovered || control.down)
                         ? "#ffffff"
                         : control.windowsCaption
                           ? projectWindow.windowsCaptionForeground
                         : control.danger && control.hovered
                           ? theme.danger
                           : control.hovered ? theme.text : theme.textSecondary
            }

            Image {
                id: macTrafficLight
                visible: control.macStyle
                anchors.fill: parent
                anchors.margins: 2
                source: {
                    var state = control.down ? "press" : control.hovered ? "hover" : "normal"
                    if (control.kind === "close") {
                        return "qrc:/assets/traffic-lights/" + (state === "normal" ? "1-close-1-normal.svg" : state === "hover" ? "2-close-2-hover.svg" : "2-close-3-press.svg")
                    }
                    if (control.kind === "minus") {
                        return "qrc:/assets/traffic-lights/" + (state === "normal" ? "2-minimize-1-normal.svg" : state === "hover" ? "2-minimize-2-hover.svg" : "2-minimize-3-press.svg")
                    }
                    return "qrc:/assets/traffic-lights/" + (state === "normal" ? "3-maximize-1-normal.svg" : state === "hover" ? "3-maximize-2-hover.svg" : "3-maximize-3-press.svg")
                }
                fillMode: Image.PreserveAspectFit
                smooth: true
                mipmap: true
                opacity: control.enabled ? 1 : 0.42
            }
        }
    }

    component WindowDragRegion: Item {
        id: dragRegion

        property bool doubleClickMaximizes: false
        property real pressX: 0
        property real pressY: 0
        property bool moveStarted: false
        readonly property int dragThreshold: 8

        MouseArea {
            anchors.fill: parent
            acceptedButtons: Qt.LeftButton
            cursorShape: Qt.ArrowCursor

            onPressed: function(mouse) {
                dragRegion.pressX = mouse.x
                dragRegion.pressY = mouse.y
                dragRegion.moveStarted = false
            }

            onPositionChanged: function(mouse) {
                if (dragRegion.moveStarted || !pressed) {
                    return
                }
                var dx = mouse.x - dragRegion.pressX
                var dy = mouse.y - dragRegion.pressY
                if (Math.sqrt(dx * dx + dy * dy) >= dragRegion.dragThreshold) {
                    dragRegion.moveStarted = true
                    windowState.startMove(dragRegion, mouse.x, mouse.y)
                }
            }

            onDoubleClicked: function(mouse) {
                if (dragRegion.doubleClickMaximizes) {
                    mouse.accepted = true
                    projectWindow.toggleMaximized()
                }
            }

            onReleased: dragRegion.moveStarted = false
            onCanceled: dragRegion.moveStarted = false
        }
    }

    Dialog {
        id: undoDialog; title: "Undo this turn's file changes?"; modal: true; anchors.centerIn: parent; width: Math.min(520, projectWindow.width - 48); standardButtons: Dialog.NoButton; closePolicy: Popup.CloseOnEscape
        background: Rectangle { color: theme.surfaceRaised; border.color: theme.borderStrong; radius: theme.radiusLarge }
        contentItem: ColumnLayout { spacing: 16
            Text { Layout.fillWidth: true; text: "SunCode will restore the files changed during this turn."; color: theme.text; font.pixelSize: theme.typeBody; wrapMode: Text.Wrap }
            Rectangle { Layout.fillWidth: true; implicitHeight: Math.min(150, restorePaths.implicitHeight + 24); color: theme.field; radius: theme.radiusMedium; border.color: theme.border; Text { id: restorePaths; anchors.fill: parent; anchors.margins: 12; text: projectWindow.pendingRestorePaths.join("\n"); color: theme.textSecondary; font.family: theme.fontMono; font.pixelSize: theme.typeLabel; wrapMode: Text.WrapAnywhere } }
            Text { Layout.fillWidth: true; text: "External side effects cannot be reversed."; color: theme.warning; font.pixelSize: theme.typeLabel; wrapMode: Text.Wrap }
            RowLayout {
                Layout.fillWidth: true
                Item { Layout.fillWidth: true }
                AppButton { theme: projectWindow.designTheme; text: "Cancel"; onClicked: undoDialog.close() }
                AppButton { theme: projectWindow.designTheme; text: "Undo changes"; tone: "primary"; onClicked: { client.restoreCheckpoint(projectWindow.pendingRestoreId); undoDialog.close() } }
            }
        }
    }

    Rectangle {
        id: windowShadowSource
        visible: false
        anchors.fill: windowBackground
        color: theme.canvas
        radius: projectWindow.windowCornerRadius
    }

    MultiEffect {
        visible: projectWindow.windowShadowVisible
        anchors.fill: windowShadowSource
        source: windowShadowSource
        autoPaddingEnabled: true
        shadowEnabled: true
        shadowColor: "#000000"
        shadowOpacity: theme.isLight ? 0.24 : 0.48
        shadowBlur: 0.58
        shadowHorizontalOffset: 0
        shadowVerticalOffset: 2
        z: -1
    }

    Rectangle {
        id: windowBackground
        anchors.fill: parent
        anchors.margins: projectWindow.windowShadowInset
        color: theme.canvas
        radius: projectWindow.windowCornerRadius
        clip: projectWindow.roundedWindowChrome

        WindowDragRegion {
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.top: parent.top
            height: projectWindow.chromeVerticalInset
            doubleClickMaximizes: true
            z: 1
        }

        ColumnLayout {
            anchors.fill: parent
            anchors.leftMargin: projectWindow.chromeHorizontalInset
            anchors.rightMargin: projectWindow.chromeHorizontalInset
            anchors.topMargin: projectWindow.chromeVerticalInset
            anchors.bottomMargin: projectWindow.chromeVerticalInset + projectWindow.footerHeight + projectWindow.chromeGap
            spacing: projectWindow.chromeGap

            Item {
                id: toolbar
                Layout.fillWidth: true
                Layout.preferredHeight: projectWindow.titleBarHeight

                WindowDragRegion {
                    id: toolbarDragRegion
                    anchors.fill: parent
                    anchors.leftMargin: leftTitleControls.visible ? leftTitleControls.width + 24 : 12
                    anchors.rightMargin: rightTitleControls.width + 24
                    doubleClickMaximizes: true
                    z: 0
                }

                Item {
                    id: leftTitleControls
                    anchors.left: parent.left
                    anchors.leftMargin: projectWindow.isMacOS ? 10 : 2
                    anchors.verticalCenter: parent.verticalCenter
                    width: leftControls.implicitWidth
                    height: leftControls.implicitHeight
                    z: 2

                    Row {
                        id: leftControls
                        spacing: 6

                        Image {
                            visible: !projectWindow.isMacOS
                            width: 22
                            height: 22
                            source: "qrc:/assets/logo/suncode-logo-small-64.png"
                            fillMode: Image.PreserveAspectFit
                            smooth: true
                            mipmap: true
                        }

                        TitleBarButton {
                            visible: projectWindow.isMacOS
                            theme: projectWindow.designTheme
                            kind: "close"
                            macStyle: true
                            Accessible.name: "Close window"
                            onClicked: projectWindow.close()
                        }

                        TitleBarButton {
                            visible: projectWindow.isMacOS
                            theme: projectWindow.designTheme
                            kind: "minus"
                            macStyle: true
                            enabled: !projectWindow.isFullScreen
                            Accessible.name: "Minimize window"
                            onClicked: projectWindow.showMinimized()
                        }

                        TitleBarButton {
                            visible: projectWindow.isMacOS
                            theme: projectWindow.designTheme
                            kind: "maximize"
                            macStyle: true
                            Accessible.name: projectWindow.isFullScreen ? "Exit full screen" : "Enter full screen"
                            onClicked: projectWindow.toggleMaximized()
                        }
                    }
                }

                Label {
                    anchors.centerIn: parent
                    width: Math.max(0, parent.width - leftTitleControls.width - rightTitleControls.width - 120)
                    text: projectWindow.currentProjectName()
                    color: theme.text
                    font.pixelSize: projectWindow.isMacOS ? 13 : theme.typeLabel
                    font.weight: Font.Medium
                    elide: Text.ElideMiddle
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                }

                Item {
                    id: rightTitleControls
                    anchors.right: parent.right
                    anchors.rightMargin: projectWindow.isMacOS ? 8 : 0
                    anchors.verticalCenter: parent.verticalCenter
                    width: rightControls.implicitWidth
                    height: rightControls.implicitHeight
                    z: 2

                    Row {
                        id: rightControls
                        spacing: projectWindow.isMacOS ? 6 : 0

                        TitleBarButton {
                            theme: projectWindow.designTheme
                            kind: "gear"
                            Accessible.name: "Open settings"
                            onClicked: projectWindow.openSettings()
                        }

                        Item {
                            visible: !projectWindow.isMacOS
                            width: visible ? 8 : 0
                            height: 1
                        }

                        TitleBarButton {
                            visible: !projectWindow.isMacOS
                            theme: projectWindow.designTheme
                            kind: "minus"
                            windowsCaption: true
                            Accessible.name: "Minimize window"
                            onClicked: projectWindow.showMinimized()
                        }

                        TitleBarButton {
                            visible: !projectWindow.isMacOS
                            theme: projectWindow.designTheme
                            kind: projectWindow.isMaximized ? "restore" : "maximize"
                            windowsCaption: true
                            Accessible.name: projectWindow.isMaximized ? "Restore window" : "Maximize window"
                            onClicked: projectWindow.toggleMaximized()
                        }

                        TitleBarButton {
                            visible: !projectWindow.isMacOS
                            theme: projectWindow.designTheme
                            kind: "close"
                            danger: true
                            windowsCaption: true
                            Accessible.name: "Close window"
                            onClicked: projectWindow.close()
                        }
                    }
                }
            }

            RowLayout {
                Layout.fillWidth: true
                Layout.fillHeight: true
                spacing: projectWindow.chromeGap

                Item {
                    id: leftGutter
                    Layout.preferredWidth: 26
                    Layout.minimumWidth: 26
                    Layout.maximumWidth: 26
                    Layout.fillHeight: true

                    WindowDragRegion {
                        anchors.fill: parent
                        doubleClickMaximizes: false
                    }

                    SidebarToggleButton {
                        id: navigationToggle
                        anchors.top: parent.top
                        anchors.horizontalCenter: parent.horizontalCenter
                        anchors.topMargin: 10
                        width: 24
                        height: 28
                        theme: projectWindow.designTheme
                        checked: projectWindow.navigationVisible
                        side: "left"
                        Accessible.name: projectWindow.navigationVisible ? "Hide project navigation" : "Show project navigation"
                        onClicked: projectWindow.toggleNavigation()
                    }

                    Button {
                        id: gitToggle
                        anchors.top: navigationToggle.bottom
                        anchors.horizontalCenter: parent.horizontalCenter
                        anchors.topMargin: 8
                        width: 24
                        height: 28
                        padding: 0
                        checkable: true
                        checked: projectWindow.gitDrawerVisible
                        hoverEnabled: true
                        focusPolicy: Qt.TabFocus
                        Accessible.name: checked ? "Close source control" : "Open source control"
                        onClicked: projectWindow.toggleGitDrawer()

                        background: Rectangle {
                            radius: theme.radiusSmall
                            color: gitToggle.checked ? theme.surfaceActive
                                                     : gitToggle.hovered ? theme.surfaceHover : "transparent"
                            border.width: gitToggle.visualFocus ? 2 : 1
                            border.color: gitToggle.visualFocus ? theme.accent
                                                                : gitToggle.checked ? theme.accentBorder : theme.border
                        }
                        contentItem: ThemeIcon {
                            anchors.centerIn: parent
                            width: 17
                            height: 17
                            source: "qrc:/assets/icons/git-branch.svg"
                            color: gitToggle.checked ? theme.accent : theme.textSecondary
                        }
                        ToolTip.visible: gitToggle.hovered
                        ToolTip.text: gitToggle.Accessible.name
                        ToolTip.delay: 500
                    }
                }

                ColumnLayout {
                    id: workColumn
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    spacing: projectWindow.chromeGap

                    RowLayout {
                        id: primaryWorkArea
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        Layout.minimumHeight: projectWindow.gitDrawerVisible ? 220 : 0
                        spacing: projectWindow.chromeGap

                        Rectangle {
                            id: connectionPanel
                            Layout.preferredWidth: projectWindow.navigationVisible ? Math.min(272, projectWindow.width * 0.22) : 0
                            Layout.minimumWidth: 0
                            Layout.maximumWidth: Math.min(288, projectWindow.width * 0.22)
                            Layout.fillHeight: true
                            visible: projectWindow.navigationVisible
                            color: theme.sidebar
                            radius: theme.radiusLarge
                            border.color: theme.border
                            clip: true

                            ConnectionPanel {
                                anchors.fill: parent
                                cardMode: true
                                client: client
                                theme: projectWindow.designTheme
                                collapsed: false
                                pinned: projectWindow.navigationPinned
                                onCollapseRequested: projectWindow.navigationVisible = false
                                onRestoreRequested: projectWindow.navigationVisible = true
                                onPinToggled: projectWindow.navigationPinned = !projectWindow.navigationPinned
                            }
                        }

                        Rectangle {
                            id: conversationCard
                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            color: theme.workspace
                            radius: theme.radiusLarge
                            border.color: theme.border
                            clip: true

                            ConversationPanel {
                                anchors.fill: parent
                                cardMode: true
                                client: client
                                theme: projectWindow.designTheme
                                contentMaximumWidth: projectWindow.conversationContentMaximumWidth
                                onSubmitRequested: function(text) { client.submitTurn(text) }
                            }
                        }

                        Rectangle {
                            id: processPanel
                            visible: projectWindow.processVisible
                            Layout.preferredWidth: visible ? Math.min(312, projectWindow.width * 0.25) : 0
                            Layout.minimumWidth: 0
                            Layout.maximumWidth: Math.min(328, projectWindow.width * 0.25)
                            Layout.fillHeight: true
                            color: theme.inspector
                            radius: theme.radiusLarge
                            border.color: theme.border
                            clip: true

                            AgentProcessPanel {
                                anchors.fill: parent
                                cardMode: true
                                client: client
                                theme: projectWindow.designTheme
                                collapsed: false
                                onCollapseRequested: projectWindow.processVisible = false
                                onRestorePanelRequested: projectWindow.processVisible = true
                                onRestoreRequested: function(manifestId, paths) {
                                    projectWindow.pendingRestoreId = manifestId
                                    projectWindow.pendingRestorePaths = paths
                                    undoDialog.open()
                                }
                            }
                        }
                    }

                    GitDrawer {
                        id: gitDrawer
                        visible: projectWindow.gitDrawerVisible
                        Layout.fillWidth: true
                        Layout.preferredHeight: visible
                                                ? Math.min(projectWindow.gitDrawerHeight, Math.max(240, workColumn.height - 220))
                                                : 0
                        Layout.minimumHeight: visible ? 240 : 0
                        Layout.maximumHeight: visible ? Math.max(240, workColumn.height - 220) : 0
                        client: client
                        theme: projectWindow.designTheme
                        onCloseRequested: projectWindow.gitDrawerVisible = false
                        onResizeRequested: function(requestedHeight) {
                            projectWindow.gitDrawerHeight = Math.max(240, Math.min(requestedHeight, workColumn.height - 220))
                        }
                    }
                }

                Item {
                    id: rightGutter
                    Layout.preferredWidth: 26
                    Layout.minimumWidth: 26
                    Layout.maximumWidth: 26
                    Layout.fillHeight: true

                    WindowDragRegion {
                        anchors.fill: parent
                        doubleClickMaximizes: false
                    }

                    SidebarToggleButton {
                        id: processToggle
                        anchors.top: parent.top
                        anchors.horizontalCenter: parent.horizontalCenter
                        anchors.topMargin: 10
                        width: 24
                        height: 28
                        theme: projectWindow.designTheme
                        checked: projectWindow.processVisible
                        side: "right"
                        Accessible.name: projectWindow.processVisible ? "Hide agent sidebar" : "Show agent sidebar"
                        onClicked: projectWindow.processVisible = !projectWindow.processVisible
                    }
                }
            }

        }

        Item {
            id: footer
            anchors.left: parent.left
            anchors.right: parent.right
            anchors.bottom: parent.bottom
            anchors.leftMargin: projectWindow.chromeHorizontalInset
            anchors.rightMargin: projectWindow.chromeHorizontalInset
            anchors.bottomMargin: projectWindow.chromeVerticalInset
            height: projectWindow.footerHeight
            z: 2

            Rectangle {
                anchors.fill: parent
                color: theme.canvas
            }

                Button {
                    id: gitFooterSummary
                    anchors.left: parent.left
                    anchors.leftMargin: 34
                    anchors.verticalCenter: parent.verticalCenter
                    width: Math.min(implicitWidth, footer.width * 0.52)
                    height: 22
                    visible: client.projectId.length > 0
                    padding: 0
                    hoverEnabled: true
                    focusPolicy: Qt.TabFocus
                    Accessible.name: projectWindow.gitDrawerVisible ? "Close source control" : "Open source control"
                    onClicked: projectWindow.toggleGitDrawer()

                    background: Rectangle {
                        radius: theme.radiusSmall
                        color: gitFooterSummary.down ? theme.surfaceActive
                                                     : gitFooterSummary.hovered ? theme.surfaceHover : "transparent"
                        border.width: gitFooterSummary.visualFocus ? 2 : 0
                        border.color: theme.accent
                    }

                    contentItem: Row {
                        id: gitFooterRow
                        leftPadding: 5
                        rightPadding: 5
                        spacing: 8

                        ThemeIcon {
                            anchors.verticalCenter: parent.verticalCenter
                            width: 13
                            height: 13
                            source: "qrc:/assets/icons/git-branch.svg"
                            color: client.gitState === "error" ? theme.danger
                                   : client.gitState === "not_repository" ? theme.textMuted : theme.accent
                        }
                        Label {
                            anchors.verticalCenter: parent.verticalCenter
                            width: Math.min(160, implicitWidth)
                            text: {
                                if (client.gitState === "loading") return "Reading Git..."
                                if (client.gitState === "not_repository") return "Not a Git repository"
                                if (client.gitState === "error") return "Git unavailable"
                                return client.gitStatus.branch || "Detached HEAD"
                            }
                            color: client.gitState === "error" ? theme.danger : theme.textSecondary
                            font.family: theme.fontMono
                            font.pixelSize: theme.typeCaption
                            elide: Text.ElideMiddle
                        }
                        Label {
                            anchors.verticalCenter: parent.verticalCenter
                            visible: client.gitState === "ready"
                            text: (client.gitStatus.changed_files || 0) === 0
                                  ? "Clean"
                                  : (client.gitStatus.changed_files || 0) + " changed"
                            color: (client.gitStatus.changed_files || 0) === 0 ? theme.success : theme.warning
                            font.family: theme.fontMono
                            font.pixelSize: theme.typeCaption
                        }
                        Label {
                            anchors.verticalCenter: parent.verticalCenter
                            visible: client.gitState === "ready" && footer.width > 760
                            text: "+" + (client.gitStatus.additions || 0)
                            color: theme.success
                            font.family: theme.fontMono
                            font.pixelSize: theme.typeCaption
                        }
                        Label {
                            anchors.verticalCenter: parent.verticalCenter
                            visible: client.gitState === "ready" && footer.width > 760
                            text: "-" + (client.gitStatus.deletions || 0)
                            color: theme.danger
                            font.family: theme.fontMono
                            font.pixelSize: theme.typeCaption
                        }
                        Label {
                            anchors.verticalCenter: parent.verticalCenter
                            visible: client.gitState === "ready" && (client.gitStatus.conflicts || 0) > 0
                            text: (client.gitStatus.conflicts || 0) + " conflicts"
                            color: theme.danger
                            font.family: theme.fontMono
                            font.pixelSize: theme.typeCaption
                            font.weight: Font.DemiBold
                        }
                    }
                }

                RowLayout {
                    anchors.right: parent.right
                    anchors.rightMargin: 10
                    anchors.verticalCenter: parent.verticalCenter
                    spacing: 8
                    visible: client.sessionId.length > 0

                    Label {
                        Layout.maximumWidth: footer.width * 0.32
                        text: client.selectedModel
                        color: theme.textMuted
                        font.family: theme.fontMono
                        font.pixelSize: theme.typeCaption
                        elide: Text.ElideMiddle
                        horizontalAlignment: Text.AlignRight
                    }

                    Rectangle {
                        Layout.preferredWidth: 1
                        Layout.preferredHeight: 10
                        color: theme.border
                    }

                    Label {
                        text: "Session " + projectWindow.compactTokenCount(client.sessionTotalTokens) + " tokens"
                        color: theme.textSecondary
                        font.family: theme.fontMono
                        font.pixelSize: theme.typeCaption
                    }
                }
            }
    }

    WindowResizeHandles {
        anchors.fill: parent
        controller: windowState
        handleSize: projectWindow.resizeHandleSize
        z: 1000
    }
}
