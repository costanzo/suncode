import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import Suncode.Runtime

ApplicationWindow {
    id: settingsWindow
    width: 860; height: 620; minimumWidth: 700; minimumHeight: 520
    title: "Suncode"
    color: theme.canvas
    readonly property var designTheme: theme

    Theme { id: theme }
    RuntimeClient { id: settingsClient; autoSelectProject: false }
    Binding { target: theme; property: "mode"; value: settingsClient.themeMode }

    header: Rectangle {
        height: 58; color: theme.surface
        Rectangle { anchors.left: parent.left; anchors.right: parent.right; anchors.bottom: parent.bottom; height: 1; color: theme.border }
        RowLayout {
            anchors.fill: parent; anchors.leftMargin: 22; anchors.rightMargin: 22
            Label { text: "Suncode"; color: theme.text; font.pixelSize: theme.typeHeading; font.weight: Font.DemiBold }
            Item { Layout.fillWidth: true }
            AppButton { theme: settingsWindow.designTheme; compact: true; text: "Done"; tone: "primary"; onClicked: settingsWindow.close() }
        }
    }

    RowLayout {
        anchors.fill: parent; spacing: 0
        Rectangle {
            Layout.preferredWidth: 210; Layout.fillHeight: true; color: theme.sidebar
            ColumnLayout {
                anchors.fill: parent; anchors.margins: 18; spacing: 5
                Label { text: "GENERAL"; color: theme.textMuted; font.pixelSize: theme.typeCaption; font.weight: Font.DemiBold; font.letterSpacing: 1.1 }
                AppButton { Layout.fillWidth: true; theme: settingsWindow.designTheme; text: "Models & providers"; checked: true }
                AppButton { Layout.fillWidth: true; theme: settingsWindow.designTheme; text: "Appearance"; enabled: false }
                AppButton { Layout.fillWidth: true; theme: settingsWindow.designTheme; text: "Permissions"; enabled: false }
            }
        }
        ScrollView { Layout.fillWidth: true; Layout.fillHeight: true; clip: true; ColumnLayout { width: settingsWindow.width - 210; anchors.margins: 28; spacing: 18
            ColumnLayout {
                Layout.fillWidth: true
                spacing: 4
                Label { text: "Models & providers"; color: theme.text; font.pixelSize: 22; font.weight: Font.DemiBold }
                Label { text: "Choose the model used for new turns and manage local provider credentials."; color: theme.textSecondary; font.pixelSize: theme.typeBody; wrapMode: Text.Wrap }
            }
            SectionLabel { theme: settingsWindow.designTheme; text: "DEFAULT MODEL" }
            ComboBox { Layout.fillWidth: true; model: ["DeepSeek"]; currentIndex: 0; font.pixelSize: theme.typeBody }
            Text { Layout.fillWidth: true; text: "Provider selection is limited to providers registered by the local runtime."; color: theme.textMuted; font.pixelSize: theme.typeLabel; wrapMode: Text.Wrap }
            ComboBox { id: modelSelector; Layout.fillWidth: true; model: settingsClient.models; textRole: "id"; currentIndex: Math.max(0, findModel(settingsClient.selectedModel)); font.pixelSize: theme.typeBody; onActivated: { settingsClient.selectedModel = currentText; settingsClient.saveUserSetting("default_model", currentText) } }
            Text { Layout.fillWidth: true; text: "The selected model is used when a project window submits a new turn."; color: theme.textMuted; font.pixelSize: theme.typeLabel; wrapMode: Text.Wrap }
            Rectangle { Layout.fillWidth: true; height: 1; color: theme.border; Layout.topMargin: 4; Layout.bottomMargin: 4 }
            SectionLabel { theme: settingsWindow.designTheme; text: "APPEARANCE" }
            RowLayout {
                Layout.fillWidth: true
                Label { text: "Theme"; color: theme.text; font.pixelSize: theme.typeBody; font.weight: Font.DemiBold }
                Item { Layout.fillWidth: true }
                ComboBox {
                    id: themeSelector
                    Layout.preferredWidth: 140
                    model: ["Dark", "Light"]
                    currentIndex: settingsClient.themeMode === "light" ? 1 : 0
                    onActivated: {
                        var value = currentIndex === 1 ? "light" : "dark"
                        settingsClient.themeMode = value
                        settingsClient.saveUserSetting("theme_mode", value)
                    }
                }
            }
            Text { Layout.fillWidth: true; text: "The theme applies across every open window."; color: theme.textMuted; font.pixelSize: theme.typeLabel; wrapMode: Text.Wrap }
            SectionLabel { theme: settingsWindow.designTheme; text: "DEEPSEEK" }
            Text { Layout.fillWidth: true; text: settingsClient.deepSeekConfigured ? "API key configured in the OS credential store." : "No API key configured."; color: settingsClient.deepSeekConfigured ? theme.success : theme.warning; font.pixelSize: theme.typeBody; wrapMode: Text.Wrap }
            AppField { id: apiKey; Layout.fillWidth: true; theme: settingsWindow.designTheme; placeholderText: "Paste DeepSeek API key"; echoMode: TextInput.Password }
            RowLayout {
                Layout.fillWidth: true
                AppButton { theme: settingsWindow.designTheme; text: "Save key"; tone: "primary"; enabled: apiKey.text.trim().length > 0; onClicked: { settingsClient.saveDeepSeekApiKey(apiKey.text); apiKey.clear() } }
                AppButton { theme: settingsWindow.designTheme; text: "Remove key"; tone: "danger"; enabled: settingsClient.deepSeekConfigured; onClicked: settingsClient.removeDeepSeekApiKey() }
                Item { Layout.fillWidth: true }
            }
            Item { Layout.fillHeight: true }
        } }
    }

    function findModel(modelId) { for (var i = 0; i < settingsClient.models.length; i++) if (settingsClient.models[i].id === modelId) return i; return 0 }
    Component.onCompleted: settingsClient.connectToRuntime()
}
