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
    signal restoreRequested(string manifestId, var paths)
    signal collapseRequested()
    signal restorePanelRequested()
    signal pinToggled()

    // Keep the review bay aligned with the conversation surface in every window.
    Layout.fillHeight: true
    color: theme.inspector
    z: 2
    clip: true
    layer.enabled: true
    layer.effect: MultiEffect {
        shadowEnabled: true
        shadowColor: "#000000"
        shadowOpacity: 0.2
        shadowBlur: 0.3
        shadowHorizontalOffset: -3
        shadowVerticalOffset: 0
    }

    Rectangle { anchors.left: parent.left; anchors.top: parent.top; anchors.bottom: parent.bottom; width: 1; color: theme.border }

    SidebarRail {
        visible: root.collapsed
        anchors.fill: parent
        side: "right"
        label: "Open agent sidebar"
        theme: root.theme
        onClicked: root.restorePanelRequested()
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
        anchors.fill: parent; anchors.margins: theme.panelPadding; clip: true
        ScrollBar.horizontal.policy: ScrollBar.AlwaysOff
        ColumnLayout {
            width: root.width - theme.panelPadding * 2; spacing: 12
            RowLayout {
                Layout.fillWidth: true
                Label { text: "AGENT PROCESSES"; color: theme.textMuted; font.pixelSize: theme.typeCaption; font.weight: Font.DemiBold; font.letterSpacing: 1.1 }
                Item { Layout.fillWidth: true }
                Rectangle { width: 7; height: 7; radius: 4; color: client.activeTurnId.length > 0 ? theme.accent : theme.textMuted }
                SidebarLockButton { theme: root.theme; locked: root.pinned; onClicked: root.pinToggled() }
                SidebarToggleButton { theme: root.theme; checked: true; side: "right"; onClicked: root.collapseRequested() }
            }
            Text { Layout.fillWidth: true; text: client.activeTurnId.length > 0 ? "1 active process" : "No active process"; color: theme.text; font.pixelSize: theme.typeTitle; font.weight: Font.DemiBold }
            Rectangle {
                Layout.fillWidth: true; implicitHeight: processColumn.implicitHeight + 26; color: client.activeTurnId.length > 0 ? theme.accentSurface : theme.surfaceRaised; border.color: client.activeTurnId.length > 0 ? theme.accentBorder : theme.border; radius: theme.radiusMedium
                ColumnLayout { id: processColumn; anchors.fill: parent; anchors.margins: 13; spacing: 8
                    RowLayout {
                        Layout.fillWidth: true
                        Rectangle { width: 7; height: 7; radius: 4; color: client.activeTurnId.length > 0 ? theme.accent : theme.textMuted }
                        Label { text: "Agent loop"; color: theme.text; font.pixelSize: theme.typeBody; font.weight: Font.DemiBold }
                        Item { Layout.fillWidth: true }
                        Label { text: client.activeTurnId.length > 0 ? "Running" : "Idle"; color: client.activeTurnId.length > 0 ? theme.accent : theme.textMuted; font.pixelSize: theme.typeCaption }
                    }
                    Text { Layout.fillWidth: true; text: client.activeTurnId.length > 0 ? "Turn " + client.activeTurnId : "Waiting for a new turn"; color: theme.textSecondary; font.pixelSize: theme.typeLabel; elide: Text.ElideMiddle }
                    Text { Layout.fillWidth: true; text: "Model  " + client.selectedModel; color: theme.textMuted; font.pixelSize: theme.typeCaption; elide: Text.ElideMiddle }
                    Text { Layout.fillWidth: true; text: client.activities.length > 0 ? "Latest  " + client.activities[client.activities.length - 1].text : "No tool activity yet"; color: theme.textSecondary; font.pixelSize: theme.typeCaption; wrapMode: Text.Wrap; maximumLineCount: 3; elide: Text.ElideRight }
                }
            }
            Text { Layout.fillWidth: true; text: "The current runtime exposes one agent loop. Tool activity and approval state appear here as the turn progresses."; color: theme.textMuted; font.pixelSize: theme.typeLabel; wrapMode: Text.Wrap; lineHeight: 1.25 }
            Rectangle { Layout.fillWidth: true; height: 1; color: theme.border; Layout.topMargin: 4; Layout.bottomMargin: 4 }
            SectionLabel { theme: root.theme; text: "REVIEW QUEUE" }
            Rectangle {
                visible: Object.keys(client.pendingApproval).length > 0
                Layout.fillWidth: true; implicitHeight: approvalColumn.implicitHeight + 26; color: theme.warningSurface; border.color: theme.warningBorder; radius: theme.radiusMedium
                ColumnLayout { id: approvalColumn; anchors.fill: parent; anchors.margins: 13; spacing: 8
                    Label { text: "Approval required"; color: theme.warning; font.pixelSize: theme.typeLabel; font.weight: Font.DemiBold }
                    Text { Layout.fillWidth: true; text: client.pendingApproval.operation + "\n" + JSON.stringify(client.pendingApproval.arguments); color: theme.text; font.pixelSize: theme.typeCaption; wrapMode: Text.WrapAnywhere }
                    RowLayout {
                        Layout.fillWidth: true
                        AppButton { Layout.fillWidth: true; theme: root.theme; compact: true; text: "Approve once"; tone: "primary"; onClicked: client.resolveApproval("allow_once") }
                        AppButton { Layout.fillWidth: true; theme: root.theme; compact: true; text: "Deny"; tone: "danger"; onClicked: client.resolveApproval("deny") }
                    }
                }
            }
            Text { visible: Object.keys(client.pendingApproval).length === 0; Layout.fillWidth: true; text: "No pending approvals"; color: theme.textMuted; font.pixelSize: theme.typeLabel }
            ListView {
                Layout.fillWidth: true; Layout.preferredHeight: Math.min(180, contentHeight); model: client.checkpoints; clip: true; spacing: 6
                delegate: Rectangle {
                    required property var modelData
                    width: ListView.view.width; height: checkpointBody.implicitHeight + 18; color: theme.surfaceRaised; radius: theme.radiusSmall; border.color: theme.border
                    ColumnLayout { id: checkpointBody; anchors.fill: parent; anchors.margins: 9; spacing: 5
                        RowLayout {
                            Layout.fillWidth: true
                            Label { Layout.fillWidth: true; text: modelData.paths.length + " file" + (modelData.paths.length === 1 ? "" : "s"); color: theme.text; font.pixelSize: theme.typeLabel }
                            Label { text: modelData.status; color: modelData.status === "available" ? theme.success : theme.warning; font.pixelSize: theme.typeCaption }
                        }
                        Text { Layout.fillWidth: true; text: modelData.paths.join("\n"); color: theme.textMuted; font.pixelSize: theme.typeCaption; wrapMode: Text.WrapAnywhere; maximumLineCount: 2; elide: Text.ElideRight }
                        AppButton { theme: root.theme; compact: true; text: "Undo"; enabled: modelData.status === "available" || modelData.status === "conflict" || modelData.status === "partial"; onClicked: root.restoreRequested(modelData.manifestId, modelData.paths) }
                    }
                }
            }
            Item { Layout.fillHeight: true }
        }
    }
}
