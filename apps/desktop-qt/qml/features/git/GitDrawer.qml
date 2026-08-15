import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../../shared/components"

Rectangle {
    id: root

    property var client
    property var theme
    property string scope: "all"
    property string selectedPath: ""
    property string filterText: ""
    property real resizeStartHeight: 0
    readonly property var statusFiles: client.gitStatus.files || []
    readonly property var filteredFiles: {
        var result = []
        var query = filterText.trim().toLowerCase()
        for (var index = 0; index < statusFiles.length; index++) {
            var file = statusFiles[index]
            var matchesScope = scope === "all"
                    || (scope === "staged" && file.staged)
                    || (scope === "unstaged" && file.unstaged)
            if (matchesScope && (query.length === 0 || String(file.path).toLowerCase().indexOf(query) >= 0)) {
                result.push(file)
            }
        }
        return result
    }

    signal closeRequested()
    signal resizeRequested(real requestedHeight)

    function selectFile(path) {
        if (!path || path.length === 0) {
            selectedPath = ""
            return
        }
        selectedPath = path
        client.loadGitDiff(scope, path)
    }

    function ensureSelection() {
        for (var index = 0; index < filteredFiles.length; index++) {
            if (filteredFiles[index].path === selectedPath) {
                client.loadGitDiff(scope, selectedPath)
                return
            }
        }
        selectFile(filteredFiles.length > 0 ? filteredFiles[0].path : "")
    }

    function copyPatch() {
        if (!client.gitDiff.patch || client.gitDiff.patch.length === 0) {
            return
        }
        clipboardProxy.selectAll()
        clipboardProxy.copy()
        clipboardProxy.deselect()
    }

    function statusLetter(file) {
        if (file.conflicted) return "!"
        if (file.status === "added" || file.status === "untracked") return "A"
        if (file.status === "deleted") return "D"
        if (file.status === "renamed") return "R"
        if (file.status === "typechange") return "T"
        return "M"
    }

    function statusColor(file) {
        if (file.conflicted || file.status === "deleted") return theme.danger
        if (file.status === "added" || file.status === "untracked") return theme.success
        return theme.warning
    }

    component IconButton: Button {
        id: iconButton
        property url iconSource
        property string toolTipText: ""

        implicitWidth: 26
        implicitHeight: 26
        padding: 0
        hoverEnabled: true
        focusPolicy: Qt.TabFocus

        background: Rectangle {
            radius: root.theme.radiusSmall
            color: iconButton.down ? root.theme.surfaceActive
                                   : iconButton.hovered ? root.theme.surfaceHover : "transparent"
            border.width: iconButton.visualFocus ? 2 : 0
            border.color: root.theme.accent
        }
        contentItem: ThemeIcon {
            anchors.centerIn: parent
            width: 14
            height: 14
            source: iconButton.iconSource
            color: iconButton.enabled
                   ? (iconButton.hovered ? root.theme.text : root.theme.textSecondary)
                   : root.theme.textDisabled
        }
        ToolTip.visible: iconButton.hovered && iconButton.toolTipText.length > 0
        ToolTip.text: iconButton.toolTipText
        ToolTip.delay: 500
    }

    component ScopeButton: Button {
        id: scopeButton
        property string scopeValue

        implicitWidth: 72
        implicitHeight: 26
        checkable: true
        checked: root.scope === scopeValue
        hoverEnabled: true
        focusPolicy: Qt.TabFocus
        padding: 0
        onClicked: root.scope = scopeValue

        contentItem: Text {
            text: scopeButton.text
            color: scopeButton.checked ? root.theme.text : root.theme.textSecondary
            font.family: root.theme.fontUi
            font.pixelSize: root.theme.typeCaption
            font.weight: scopeButton.checked ? Font.DemiBold : Font.Medium
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
        }
        background: Rectangle {
            radius: root.theme.radiusSmall - 1
            color: scopeButton.checked ? root.theme.surfaceActive
                                       : scopeButton.hovered ? root.theme.surfaceHover : "transparent"
            border.width: scopeButton.visualFocus ? 2 : 0
            border.color: root.theme.accent
        }
    }

    color: theme.surface
    radius: theme.radiusSmall
    border.color: theme.border
    clip: true

    onScopeChanged: Qt.callLater(ensureSelection)
    onFilteredFilesChanged: Qt.callLater(ensureSelection)

    Connections {
        target: client
        function onGitStatusChanged() { Qt.callLater(root.ensureSelection) }
    }

    Item {
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        height: 8
        z: 4

        HoverHandler {
            cursorShape: Qt.SizeVerCursor
        }
        DragHandler {
            id: resizeHandler
            target: null
            xAxis.enabled: false
            yAxis.enabled: true
            acceptedButtons: Qt.LeftButton
            cursorShape: Qt.SizeVerCursor
            onActiveChanged: {
                if (active) root.resizeStartHeight = root.height
            }
            onTranslationChanged: {
                if (active) root.resizeRequested(root.resizeStartHeight - translation.y)
            }
        }
    }

    TextArea {
        id: clipboardProxy
        visible: false
        text: client.gitDiff.patch || ""
    }

    ColumnLayout {
        anchors.fill: parent
        spacing: 0

        RowLayout {
            Layout.fillWidth: true
            Layout.preferredHeight: 36
            Layout.minimumHeight: 36
            Layout.leftMargin: 12
            Layout.rightMargin: 8
            spacing: 8

            ThemeIcon {
                Layout.preferredWidth: 16
                Layout.preferredHeight: 16
                source: "qrc:/assets/icons/git-branch.svg"
                color: theme.accent
            }

            Label {
                Layout.maximumWidth: 190
                text: client.gitState === "ready"
                      ? (client.gitStatus.branch || "Detached HEAD")
                      : "Source control"
                color: theme.text
                font.family: theme.fontMono
                font.pixelSize: theme.typeLabel
                font.weight: Font.DemiBold
                elide: Text.ElideMiddle
            }

            Rectangle {
                Layout.preferredWidth: 1
                Layout.preferredHeight: 18
                color: theme.border
            }

            Row {
                Layout.preferredHeight: 28
                spacing: 0

                Rectangle {
                    width: scopeButtons.width + 2
                    height: 28
                    radius: theme.radiusSmall
                    color: theme.field
                    border.color: theme.border

                    Row {
                        id: scopeButtons
                        anchors.centerIn: parent
                        ScopeButton { text: "All"; scopeValue: "all" }
                        ScopeButton { text: "Staged"; scopeValue: "staged" }
                        ScopeButton { text: "Unstaged"; scopeValue: "unstaged" }
                    }
                }
            }

            Item { Layout.fillWidth: true }

            AppField {
                id: filterField
                Layout.preferredWidth: Math.min(210, Math.max(130, root.width * 0.18))
                Layout.preferredHeight: 28
                theme: root.theme
                placeholderText: "Filter changed files"
                font.pixelSize: theme.typeLabel
                onTextChanged: root.filterText = text
            }

            IconButton {
                iconSource: "qrc:/assets/icons/refresh.svg"
                toolTipText: "Refresh Git status"
                enabled: client.gitState !== "loading"
                Accessible.name: toolTipText
                onClicked: client.refreshGitStatus()
            }

            IconButton {
                iconSource: "qrc:/assets/icons/copy.svg"
                toolTipText: "Copy patch"
                enabled: client.gitDiffState === "ready" && (client.gitDiff.patch || "").length > 0
                Accessible.name: toolTipText
                onClicked: root.copyPatch()
            }

            IconButton {
                iconSource: "qrc:/assets/icons/close.svg"
                toolTipText: "Close source control"
                Accessible.name: toolTipText
                onClicked: root.closeRequested()
            }
        }

        Rectangle {
            Layout.fillWidth: true
            Layout.preferredHeight: 1
            color: theme.border
        }

        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 0

            Rectangle {
                Layout.preferredWidth: Math.min(300, Math.max(228, root.width * 0.28))
                Layout.minimumWidth: 228
                Layout.fillHeight: true
                color: theme.sidebar

                ColumnLayout {
                    anchors.fill: parent
                    spacing: 0

                    RowLayout {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 30
                        Layout.leftMargin: 12
                        Layout.rightMargin: 10

                        Label {
                            text: root.filteredFiles.length + (root.filteredFiles.length === 1 ? " file" : " files")
                            color: theme.textMuted
                            font.family: theme.fontUi
                            font.pixelSize: theme.typeCaption
                            font.weight: Font.DemiBold
                        }
                        Item { Layout.fillWidth: true }
                        Label {
                            visible: client.gitStatus.truncated === true
                            text: "LIMITED"
                            color: theme.warning
                            font.family: theme.fontMono
                            font.pixelSize: theme.typeCaption
                        }
                    }

                    ListView {
                        id: fileList
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        model: root.filteredFiles
                        clip: true
                        boundsBehavior: Flickable.StopAtBounds
                        ScrollBar.vertical: ScrollBar {}

                        delegate: ItemDelegate {
                            id: fileDelegate
                            required property var modelData
                            width: fileList.width
                            height: 42
                            hoverEnabled: true
                            focusPolicy: Qt.TabFocus
                            highlighted: root.selectedPath === modelData.path
                            onClicked: root.selectFile(modelData.path)

                            background: Rectangle {
                                color: fileDelegate.highlighted ? theme.surfaceActive
                                                              : fileDelegate.hovered ? theme.surfaceHover : "transparent"
                                border.width: fileDelegate.visualFocus ? 2 : 0
                                border.color: theme.accent
                            }

                            contentItem: RowLayout {
                                spacing: 8

                                Label {
                                    Layout.preferredWidth: 18
                                    text: root.statusLetter(fileDelegate.modelData)
                                    color: root.statusColor(fileDelegate.modelData)
                                    font.family: theme.fontMono
                                    font.pixelSize: theme.typeLabel
                                    font.weight: Font.Bold
                                    horizontalAlignment: Text.AlignHCenter
                                }

                                ColumnLayout {
                                    Layout.fillWidth: true
                                    spacing: 0

                                    Label {
                                        Layout.fillWidth: true
                                        text: fileDelegate.modelData.path
                                        textFormat: Text.PlainText
                                        color: theme.text
                                        font.family: theme.fontMono
                                        font.pixelSize: theme.typeCaption
                                        elide: Text.ElideMiddle
                                    }
                                    Label {
                                        Layout.fillWidth: true
                                        visible: (fileDelegate.modelData.old_path || "").length > 0
                                        text: "from " + fileDelegate.modelData.old_path
                                        textFormat: Text.PlainText
                                        color: theme.textMuted
                                        font.family: theme.fontMono
                                        font.pixelSize: 10
                                        elide: Text.ElideMiddle
                                    }
                                }

                                Label {
                                    text: fileDelegate.modelData.binary
                                          ? "BIN"
                                          : "+" + fileDelegate.modelData.additions + "  -" + fileDelegate.modelData.deletions
                                    color: fileDelegate.modelData.binary ? theme.textMuted : theme.textSecondary
                                    font.family: theme.fontMono
                                    font.pixelSize: 10
                                }
                            }
                        }
                    }
                }
            }

            Rectangle {
                Layout.preferredWidth: 1
                Layout.fillHeight: true
                color: theme.border
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.fillHeight: true
                color: theme.workspace

                ColumnLayout {
                    anchors.fill: parent
                    spacing: 0

                    RowLayout {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 30
                        Layout.leftMargin: 12
                        Layout.rightMargin: 12
                        spacing: 10

                        Label {
                            Layout.fillWidth: true
                            text: root.selectedPath || "No file selected"
                            textFormat: Text.PlainText
                            color: theme.textSecondary
                            font.family: theme.fontMono
                            font.pixelSize: theme.typeCaption
                            elide: Text.ElideMiddle
                        }
                        Label {
                            visible: client.gitDiffState === "ready" && client.gitDiff.binary !== true
                            text: "+" + (client.gitDiff.additions || 0)
                            color: theme.success
                            font.family: theme.fontMono
                            font.pixelSize: theme.typeCaption
                        }
                        Label {
                            visible: client.gitDiffState === "ready" && client.gitDiff.binary !== true
                            text: "-" + (client.gitDiff.deletions || 0)
                            color: theme.danger
                            font.family: theme.fontMono
                            font.pixelSize: theme.typeCaption
                        }
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: 1
                        color: theme.border
                    }

                    ListView {
                        id: diffList
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        visible: client.gitDiffState === "ready"
                                 && client.gitDiff.binary !== true
                                 && client.gitDiffRows.length > 0
                        model: client.gitDiffRows
                        clip: true
                        boundsBehavior: Flickable.StopAtBounds
                        ScrollBar.vertical: ScrollBar {}
                        ScrollBar.horizontal: ScrollBar {}

                        delegate: Rectangle {
                            id: diffRow
                            required property var modelData
                            width: Math.max(diffList.width, lineLayout.implicitWidth)
                            height: 22
                            color: {
                                if (modelData.kind === "addition") return theme.successSurface
                                if (modelData.kind === "deletion") return theme.dangerSurface
                                if (modelData.kind === "hunk") return theme.accentSurface
                                return "transparent"
                            }

                            Row {
                                id: lineLayout
                                height: parent.height

                                Label {
                                    width: 44
                                    height: parent.height
                                    text: diffRow.modelData.kind === "hunk" ? "" : (diffRow.modelData.old_line || "")
                                    color: theme.textMuted
                                    font.family: theme.fontMono
                                    font.pixelSize: 10
                                    horizontalAlignment: Text.AlignRight
                                    verticalAlignment: Text.AlignVCenter
                                    rightPadding: 8
                                }
                                Label {
                                    width: 44
                                    height: parent.height
                                    text: diffRow.modelData.kind === "hunk" ? "" : (diffRow.modelData.new_line || "")
                                    color: theme.textMuted
                                    font.family: theme.fontMono
                                    font.pixelSize: 10
                                    horizontalAlignment: Text.AlignRight
                                    verticalAlignment: Text.AlignVCenter
                                    rightPadding: 8
                                }
                                Rectangle { width: 1; height: parent.height; color: theme.border }
                                Label {
                                    height: parent.height
                                    leftPadding: 10
                                    rightPadding: 16
                                    text: diffRow.modelData.kind === "hunk"
                                          ? diffRow.modelData.text
                                          : (diffRow.modelData.kind === "addition" ? "+"
                                             : diffRow.modelData.kind === "deletion" ? "-" : " ") + diffRow.modelData.text
                                    textFormat: Text.PlainText
                                    color: diffRow.modelData.kind === "hunk" ? theme.accent : theme.text
                                    font.family: theme.fontMono
                                    font.pixelSize: theme.typeCaption
                                    verticalAlignment: Text.AlignVCenter
                                }
                            }
                        }
                    }

                    Item {
                        Layout.fillWidth: true
                        Layout.fillHeight: true
                        visible: !diffList.visible

                        Column {
                            anchors.centerIn: parent
                            width: Math.min(parent.width - 48, 420)
                            spacing: 8

                            BusyIndicator {
                                anchors.horizontalCenter: parent.horizontalCenter
                                visible: client.gitState === "loading" || client.gitDiffState === "loading"
                                running: visible
                                width: 28
                                height: 28
                            }
                            Label {
                                width: parent.width
                                text: {
                                    if (client.gitState === "loading") return "Reading repository changes..."
                                    if (client.gitState === "not_repository") return "This project is not inside a Git repository."
                                    if (client.gitState === "error") return client.gitError || "Git status is unavailable."
                                    if (root.filteredFiles.length === 0 && root.filterText.length > 0) return "No changed files match this filter."
                                    if (root.filteredFiles.length === 0 && client.gitState === "ready") return root.scope === "all" ? "Working tree clean." : "No " + root.scope + " changes."
                                    if (client.gitDiffState === "loading") return "Loading diff..."
                                    if (client.gitDiffState === "error") return client.gitError || "This diff is unavailable."
                                    if (client.gitDiff.binary === true) return "Binary files cannot be displayed as text."
                                    return "Select a changed file to inspect its diff."
                                }
                                color: client.gitState === "error" || client.gitDiffState === "error" ? theme.danger : theme.textSecondary
                                font.family: theme.fontUi
                                font.pixelSize: theme.typeBody
                                horizontalAlignment: Text.AlignHCenter
                                wrapMode: Text.Wrap
                            }
                        }
                    }

                    Rectangle {
                        Layout.fillWidth: true
                        Layout.preferredHeight: visible ? 24 : 0
                        visible: client.gitDiff.truncated === true
                        color: theme.warningSurface

                        Label {
                            anchors.centerIn: parent
                            text: "Diff truncated at the review limit"
                            color: theme.warning
                            font.family: theme.fontUi
                            font.pixelSize: theme.typeCaption
                        }
                    }
                }
            }
        }
    }

    Item {
        anchors.left: parent.left
        anchors.bottom: parent.bottom
        width: root.radius
        height: root.radius
        clip: true
        z: 3

        Rectangle {
            anchors.fill: parent
            color: theme.canvas
        }
        Rectangle {
            x: 0
            y: -root.radius
            width: root.radius * 2
            height: root.radius * 2
            radius: root.radius
            color: theme.sidebar
        }
    }

    Item {
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        width: root.radius
        height: root.radius
        clip: true
        z: 3

        Rectangle {
            anchors.fill: parent
            color: theme.canvas
        }
        Rectangle {
            x: -root.radius
            y: -root.radius
            width: root.radius * 2
            height: root.radius * 2
            radius: root.radius
            color: theme.workspace
        }
    }
}
