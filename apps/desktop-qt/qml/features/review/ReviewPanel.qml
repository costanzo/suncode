import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import "../../shared/components"

Rectangle {
    id: root
    property var client
    property var theme
    signal restoreRequested(string manifestId, var paths)

    Layout.fillHeight: true

    color: theme.inspector
    Rectangle { anchors.top: parent.top; anchors.bottom: parent.bottom; anchors.left: parent.left; width: 1; color: theme.border }

    ScrollView {
        anchors.fill: parent
        anchors.leftMargin: theme.panelPadding
        anchors.rightMargin: theme.panelPadding
        anchors.topMargin: 18
        anchors.bottomMargin: 14
        clip: true
        ScrollBar.horizontal.policy: ScrollBar.AlwaysOff

        ColumnLayout {
            width: root.width - theme.panelPadding * 2
            spacing: 12

            RowLayout {
                Layout.fillWidth: true
                Label { text: "REVIEW"; color: theme.textMuted; font.pixelSize: theme.typeCaption; font.weight: Font.DemiBold; font.letterSpacing: 1.1 }
                Item { Layout.fillWidth: true }
                Rectangle { width: 7; height: 7; radius: 4; color: Object.keys(client.pendingApproval).length > 0 ? theme.warning : theme.textMuted }
            }

            Rectangle {
                visible: Object.keys(client.pendingApproval).length > 0
                Layout.fillWidth: true
                implicitHeight: approvalColumn.implicitHeight + 28
                color: theme.warningSurface
                border.color: theme.warningBorder
                radius: theme.radiusMedium

                ColumnLayout {
                    id: approvalColumn
                    anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top
                    anchors.margins: 13; spacing: 10
                    RowLayout {
                        Layout.fillWidth: true
                        Rectangle { width: 7; height: 7; radius: 4; color: theme.warning }
                        Label { text: "Approval required"; color: theme.warning; font.pixelSize: theme.typeLabel; font.weight: Font.DemiBold }
                        Item { Layout.fillWidth: true }
                    }
                    Text { Layout.fillWidth: true; text: client.pendingApproval.operation + "\n" + JSON.stringify(client.pendingApproval.arguments); color: theme.text; wrapMode: Text.Wrap; textFormat: Text.PlainText; font.family: theme.fontMono; font.pixelSize: theme.typeLabel; lineHeight: 1.2 }
                    RowLayout {
                        Layout.fillWidth: true
                        AppButton { Layout.fillWidth: true; theme: root.theme; compact: true; text: "Approve once"; tone: "primary"; onClicked: client.resolveApproval("allow_once") }
                        AppButton { Layout.fillWidth: true; theme: root.theme; compact: true; text: "Deny"; tone: "danger"; onClicked: client.resolveApproval("deny") }
                    }
                }
            }

            Text { visible: Object.keys(client.pendingApproval).length === 0; Layout.fillWidth: true; text: "No actions are waiting for approval."; color: theme.textMuted; font.pixelSize: theme.typeLabel; wrapMode: Text.Wrap }

            Rectangle { Layout.fillWidth: true; height: 1; color: theme.border; Layout.topMargin: 4; Layout.bottomMargin: 4 }
            SectionLabel { theme: root.theme; text: "TURN CHANGES" }

            ListView {
                Layout.fillWidth: true
                Layout.preferredHeight: Math.min(250, contentHeight)
                model: client.checkpoints
                spacing: 8
                clip: true

                delegate: Rectangle {
                    required property var modelData
                    width: ListView.view.width
                    height: checkpointBody.implicitHeight + 20
                    color: theme.surfaceRaised
                    border.color: theme.border
                    radius: theme.radiusMedium

                    ColumnLayout {
                        id: checkpointBody
                        anchors.left: parent.left; anchors.right: parent.right; anchors.top: parent.top
                        anchors.margins: 10; spacing: 7
                        RowLayout {
                            Layout.fillWidth: true
                            Label { Layout.fillWidth: true; text: modelData.paths.length + (modelData.paths.length === 1 ? " file" : " files"); color: theme.text; font.pixelSize: theme.typeLabel; font.weight: Font.DemiBold }
                            Label { text: modelData.status; color: modelData.status === "available" ? theme.success : modelData.status === "conflict" || modelData.status === "partial" ? theme.warning : theme.textMuted; font.pixelSize: theme.typeCaption }
                        }
                        Text { Layout.fillWidth: true; text: modelData.paths.join("\n"); color: theme.textSecondary; wrapMode: Text.WrapAnywhere; textFormat: Text.PlainText; font.family: theme.fontMono; font.pixelSize: theme.typeCaption; lineHeight: 1.15 }
                        AppButton { theme: root.theme; compact: true; text: modelData.status === "available" ? "Undo" : "Review undo"; enabled: modelData.status === "available" || modelData.status === "conflict" || modelData.status === "partial"; onClicked: root.restoreRequested(modelData.manifestId, modelData.paths) }
                    }
                }
            }

            Text { visible: client.checkpoints.length === 0; Layout.fillWidth: true; text: "File changes from completed turns will appear here."; color: theme.textMuted; font.pixelSize: theme.typeLabel; wrapMode: Text.Wrap }

            SectionLabel { theme: root.theme; text: "FILES TOUCHED"; Layout.topMargin: 5 }
            ListView { Layout.fillWidth: true; Layout.preferredHeight: Math.min(130, contentHeight); model: client.changedPaths; clip: true; spacing: 3; delegate: Label { required property var modelData; width: ListView.view.width; text: "•  " + modelData; color: theme.textSecondary; elide: Text.ElideMiddle; font.family: theme.fontMono; font.pixelSize: theme.typeCaption } }
            Text { visible: client.changedPaths.length === 0; Layout.fillWidth: true; text: "No file mutations in this session."; color: theme.textMuted; font.pixelSize: theme.typeLabel }

            Rectangle { Layout.fillWidth: true; height: 1; color: theme.border; Layout.topMargin: 5; Layout.bottomMargin: 4 }
            SectionLabel { theme: root.theme; text: "RUNTIME" }
            Text { Layout.fillWidth: true; text: client.statusText; color: theme.textMuted; wrapMode: Text.Wrap; font.pixelSize: theme.typeLabel }
            Rectangle {
                Layout.fillWidth: true
                implicitHeight: runtimeStatus.implicitHeight + 18
                color: client.diagnostics.health && client.diagnostics.health.database.ok ? theme.successSurface : theme.warningSurface
                radius: theme.radiusSmall
                border.color: client.diagnostics.health && client.diagnostics.health.database.ok ? theme.accentBorder : theme.warningBorder
                Text { id: runtimeStatus; anchors.fill: parent; anchors.margins: 9; text: client.diagnostics.health ? "Runtime  " + client.diagnostics.health.runtime + "\nDatabase  " + (client.diagnostics.health.database.ok ? "Ready" : "Check required") : "Diagnostics unavailable"; color: client.diagnostics.health && client.diagnostics.health.database.ok ? theme.success : theme.warning; font.family: theme.fontMono; font.pixelSize: theme.typeCaption; lineHeight: 1.3 }
            }
            AppButton { Layout.fillWidth: true; theme: root.theme; compact: true; text: "Refresh diagnostics"; onClicked: client.refreshDiagnostics() }
            Item { Layout.fillHeight: true; Layout.minimumHeight: 20 }
        }
    }
}
