import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import SunCode.Runtime
import "../../shared/components"
import "../../shared/theme"

ApplicationWindow {
    id: settingsWindow

    property string selectedPage: "defaults"
    property bool providersExpanded: true
    readonly property var designTheme: theme

    width: 900
    height: 640
    minimumWidth: 720
    minimumHeight: 520
    title: "Settings"
    color: theme.canvas
    modality: Qt.ApplicationModal

    Theme { id: theme }
    RuntimeClient { id: settingsClient; autoSelectProject: false }
    Binding { target: theme; property: "mode"; value: settingsClient.themeMode }

    Shortcut {
        sequences: [StandardKey.Cancel]
        onActivated: settingsWindow.close()
    }

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

    header: Rectangle {
        height: 58
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
            anchors.leftMargin: 22
            anchors.rightMargin: 22

            Label {
                text: "Settings"
                color: theme.text
                font.pixelSize: theme.typeHeading
                font.weight: Font.DemiBold
            }
            Item { Layout.fillWidth: true }
            AppButton {
                theme: settingsWindow.designTheme
                compact: true
                text: "Done"
                tone: "primary"
                onClicked: settingsWindow.close()
            }
        }
    }

    component NavigationItem: Button {
        id: navigationItem

        property string page: ""
        property int depth: 0
        property bool folder: false
        property bool expanded: false

        Layout.fillWidth: true
        implicitHeight: 34
        hoverEnabled: true
        checkable: !folder
        checked: !folder && settingsWindow.selectedPage === page
        leftPadding: 10 + depth * 18
        rightPadding: 10
        focusPolicy: Qt.TabFocus
        Accessible.name: text
        Accessible.role: folder ? Accessible.TreeItem : Accessible.ListItem

        contentItem: RowLayout {
            spacing: 8

            ThemeIcon {
                visible: navigationItem.folder
                Layout.preferredWidth: navigationItem.folder ? 10 : 0
                Layout.preferredHeight: 10
                source: "qrc:/assets/icons/chevron-right.svg"
                color: navigationItem.checked ? theme.accent : theme.textSecondary
                rotation: navigationItem.expanded ? 90 : 0
            }

            Label {
                Layout.fillWidth: true
                text: navigationItem.text
                color: navigationItem.checked ? theme.text : (navigationItem.enabled ? theme.textSecondary : theme.textDisabled)
                font.pixelSize: theme.typeLabel
                font.weight: navigationItem.checked || navigationItem.folder ? Font.DemiBold : Font.Medium
                elide: Text.ElideRight
            }
        }

        background: Rectangle {
            radius: theme.radiusSmall
            color: navigationItem.checked ? theme.surfaceActive : (navigationItem.hovered ? theme.surfaceHover : "transparent")
            border.width: navigationItem.visualFocus ? 2 : 0
            border.color: theme.accent
        }

        onClicked: {
            if (folder) settingsWindow.providersExpanded = !settingsWindow.providersExpanded
            else settingsWindow.selectedPage = page
        }
    }

    component ProviderPage: ScrollView {
        id: providerPage
        property string providerId: ""
        property string providerTitle: ""
        property string description: ""
        property string placeholder: ""

        clip: true
        contentWidth: availableWidth

        Item {
            width: parent.width
            implicitHeight: providerContent.implicitHeight + 56

            ColumnLayout {
                id: providerContent
                x: 32
                y: 28
                width: parent.width - 64
                spacing: 18

                ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 4
                    Label { text: providerPage.providerTitle; color: theme.text; font.pixelSize: 22; font.weight: Font.DemiBold }
                    Label {
                        Layout.fillWidth: true
                        text: providerPage.description
                        color: theme.textSecondary
                        font.pixelSize: theme.typeBody
                        wrapMode: Text.Wrap
                    }
                }

                SectionLabel { theme: settingsWindow.designTheme; text: "CREDENTIAL" }
                Text {
                    Layout.fillWidth: true
                    text: settingsWindow.providerConfigured(providerPage.providerId)
                          ? "API key configured in the local runtime credential store."
                          : "No API key configured."
                    color: settingsWindow.providerConfigured(providerPage.providerId) ? theme.success : theme.warning
                    font.pixelSize: theme.typeBody
                    wrapMode: Text.Wrap
                }
                AppField {
                    id: providerApiKey
                    Layout.fillWidth: true
                    theme: settingsWindow.designTheme
                    placeholderText: providerPage.placeholder
                    echoMode: TextInput.Password
                    font.family: theme.fontMono
                }
                RowLayout {
                    Layout.fillWidth: true
                    AppButton {
                        theme: settingsWindow.designTheme
                        text: "Save key"
                        tone: "primary"
                        enabled: providerApiKey.text.trim().length > 0
                        onClicked: {
                            settingsClient.saveCredential(providerPage.providerId, providerApiKey.text)
                            providerApiKey.clear()
                        }
                    }
                    AppButton {
                        theme: settingsWindow.designTheme
                        text: "Remove key"
                        tone: "danger"
                        enabled: settingsWindow.providerConfigured(providerPage.providerId)
                        onClicked: settingsClient.removeCredential(providerPage.providerId)
                    }
                    Item { Layout.fillWidth: true }
                }

                Rectangle { Layout.fillWidth: true; Layout.preferredHeight: 1; color: theme.border; Layout.topMargin: 4; Layout.bottomMargin: 4 }
                SectionLabel { theme: settingsWindow.designTheme; text: "AVAILABLE MODELS" }
                Text {
                    Layout.fillWidth: true
                    text: settingsWindow.providerModels(providerPage.providerId)
                    color: theme.text
                    font.family: theme.fontMono
                    font.pixelSize: theme.typeBody
                    wrapMode: Text.WrapAnywhere
                }
                Text {
                    Layout.fillWidth: true
                    text: "The runtime owns provider availability and model registration."
                    color: theme.textMuted
                    font.pixelSize: theme.typeLabel
                    wrapMode: Text.Wrap
                }
                Item { Layout.fillHeight: true }
            }
        }
    }

    RowLayout {
        anchors.fill: parent
        spacing: 0

        Rectangle {
            Layout.preferredWidth: 238
            Layout.fillHeight: true
            color: theme.sidebar

            Rectangle {
                anchors.top: parent.top
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                width: 1
                color: theme.border
            }

            ColumnLayout {
                anchors.fill: parent
                anchors.leftMargin: 16
                anchors.rightMargin: 16
                anchors.topMargin: 20
                anchors.bottomMargin: 20
                spacing: 4

                SectionLabel { theme: settingsWindow.designTheme; text: "GENERAL" }
                NavigationItem { text: "Defaults"; page: "defaults" }
                NavigationItem { text: "Appearance"; page: "appearance" }

                Item { Layout.preferredHeight: 12 }
                SectionLabel { theme: settingsWindow.designTheme; text: "MODELS" }
                NavigationItem {
                    text: "Model providers"
                    folder: true
                    expanded: settingsWindow.providersExpanded
                }
                NavigationItem {
                    visible: settingsWindow.providersExpanded
                    text: "DeepSeek"
                    page: "deepseek"
                    depth: 1
                }
                NavigationItem {
                    visible: settingsWindow.providersExpanded
                    text: "Zhipu GLM"
                    page: "zhipu"
                    depth: 1
                }
                NavigationItem {
                    visible: settingsWindow.providersExpanded
                    text: "OpenAI"
                    page: "openai"
                    depth: 1
                }
                NavigationItem {
                    visible: settingsWindow.providersExpanded
                    text: "Kimi"
                    page: "kimi"
                    depth: 1
                }
                NavigationItem {
                    visible: settingsWindow.providersExpanded
                    text: "Claude"
                    page: "claude"
                    depth: 1
                }
                NavigationItem {
                    visible: settingsWindow.providersExpanded
                    text: "Gemini"
                    page: "gemini"
                    depth: 1
                }

                Item { Layout.fillHeight: true }
            }
        }

        StackLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: settingsWindow.selectedPage === "appearance" ? 1
                        : settingsWindow.selectedPage === "deepseek" ? 2
                        : settingsWindow.selectedPage === "zhipu" ? 3
                        : settingsWindow.selectedPage === "openai" ? 4
                        : settingsWindow.selectedPage === "kimi" ? 5
                        : settingsWindow.selectedPage === "claude" ? 6
                        : settingsWindow.selectedPage === "gemini" ? 7 : 0

            ScrollView {
                clip: true
                contentWidth: availableWidth

                Item {
                    width: parent.width
                    implicitHeight: defaultsContent.implicitHeight + 56

                    ColumnLayout {
                        id: defaultsContent
                        x: 32
                        y: 28
                        width: parent.width - 64
                        spacing: 18

                    ColumnLayout {
                        Layout.fillWidth: true
                        spacing: 4
                        Label { text: "Defaults"; color: theme.text; font.pixelSize: 22; font.weight: Font.DemiBold }
                        Label {
                            Layout.fillWidth: true
                            text: "Choose the model SunCode uses when a project starts a new turn."
                            color: theme.textSecondary
                            font.pixelSize: theme.typeBody
                            wrapMode: Text.Wrap
                        }
                    }

                    SectionLabel { theme: settingsWindow.designTheme; text: "DEFAULT MODEL" }
                    AppComboBox {
                        id: modelSelector
                        theme: settingsWindow.designTheme
                        Layout.fillWidth: true
                        model: settingsClient.models
                        textRole: "id"
                        currentIndex: Math.max(0, settingsWindow.findModel(settingsClient.selectedModel))
                        font.family: theme.fontMono
                        font.pixelSize: theme.typeBody
                        onActivated: {
                            settingsClient.selectedModel = currentText
                            settingsClient.saveUserSetting("default_model", currentText)
                        }
                    }
                    Text {
                        Layout.fillWidth: true
                        text: "Only models registered by the local runtime appear here."
                        color: theme.textMuted
                        font.pixelSize: theme.typeLabel
                        wrapMode: Text.Wrap
                    }
                        Item { Layout.fillHeight: true }
                    }
                }
            }

            ScrollView {
                clip: true
                contentWidth: availableWidth

                Item {
                    width: parent.width
                    implicitHeight: appearanceContent.implicitHeight + 56

                    ColumnLayout {
                        id: appearanceContent
                        x: 32
                        y: 28
                        width: parent.width - 64
                        spacing: 18

                        ColumnLayout {
                            Layout.fillWidth: true
                            spacing: 4
                            Label { text: "Appearance"; color: theme.text; font.pixelSize: 22; font.weight: Font.DemiBold }
                            Label {
                                Layout.fillWidth: true
                                text: "Adjust how SunCode looks across every open window."
                                color: theme.textSecondary
                                font.pixelSize: theme.typeBody
                                wrapMode: Text.Wrap
                            }
                        }

                        SectionLabel { theme: settingsWindow.designTheme; text: "THEME" }
                        RowLayout {
                            Layout.fillWidth: true
                            Label { text: "Color theme"; color: theme.text; font.pixelSize: theme.typeBody; font.weight: Font.DemiBold }
                            Item { Layout.fillWidth: true }
                            AppComboBox {
                                id: themeSelector
                                theme: settingsWindow.designTheme
                                Layout.preferredWidth: 160
                                model: ["Dark", "Light"]
                                currentIndex: settingsClient.themeMode === "light" ? 1 : 0
                                onActivated: {
                                    var value = currentIndex === 1 ? "light" : "dark"
                                    settingsClient.themeMode = value
                                    settingsClient.saveUserSetting("theme_mode", value)
                                }
                            }
                        }
                        Text {
                            Layout.fillWidth: true
                            text: "Changes apply immediately."
                            color: theme.textMuted
                            font.pixelSize: theme.typeLabel
                            wrapMode: Text.Wrap
                        }
                        Item { Layout.fillHeight: true }
                    }
                }
            }

            ProviderPage {
                visible: settingsWindow.selectedPage === "deepseek"
                providerId: "deepseek"
                providerTitle: "DeepSeek"
                description: "Configure the credential used by the local DeepSeek provider."
                placeholder: "Paste DeepSeek API key"
            }

            ProviderPage {
                visible: settingsWindow.selectedPage === "zhipu"
                providerId: "zhipu"
                providerTitle: "Zhipu GLM"
                description: "Configure the credential used by the local Zhipu GLM provider."
                placeholder: "Paste Zhipu API key"
            }

            ProviderPage {
                visible: settingsWindow.selectedPage === "openai"
                providerId: "openai"
                providerTitle: "OpenAI"
                description: "Configure the credential used by the local OpenAI provider."
                placeholder: "Paste OpenAI API key"
            }

            ProviderPage {
                visible: settingsWindow.selectedPage === "kimi"
                providerId: "kimi"
                providerTitle: "Kimi"
                description: "Configure the credential used by the local Kimi provider."
                placeholder: "Paste Kimi API key"
            }

            ProviderPage {
                visible: settingsWindow.selectedPage === "claude"
                providerId: "claude"
                providerTitle: "Claude"
                description: "Configure the credential used by the local Claude provider."
                placeholder: "Paste Anthropic API key"
            }

            ProviderPage {
                visible: settingsWindow.selectedPage === "gemini"
                providerId: "gemini"
                providerTitle: "Gemini"
                description: "Configure the credential used by the local Gemini provider."
                placeholder: "Paste Gemini API key"
            }
        }
    }

    function findModel(modelId) {
        for (var index = 0; index < settingsClient.models.length; index++) {
            if (settingsClient.models[index].id === modelId) return index
        }
        return 0
    }

    function providerConfigured(providerId) {
        for (var index = 0; index < settingsClient.credentials.length; index++) {
            var credential = settingsClient.credentials[index]
            if (credential.provider === providerId) {
                return credential.configured
            }
        }
        return false
    }

    function providerModels(providerId) {
        var ids = []
        for (var index = 0; index < settingsClient.models.length; index++) {
            var model = settingsClient.models[index]
            if (model.provider === providerId) {
                ids.push(model.id + "  " + (model.availability === "configured" ? "configured" : "needs key"))
            }
        }
        return ids.length > 0 ? ids.join("\n") : "No models available"
    }

    Component.onCompleted: settingsClient.connectToRuntime()
}
