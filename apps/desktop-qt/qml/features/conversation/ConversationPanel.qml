import QtQuick
import QtQuick.Controls
import QtQuick.Effects
import QtQuick.Layouts
import "../../shared/components"
import "../../shared/theme"

Item {
    id: root
    property var client
    property var theme
    property bool pendingInitialScroll: false
    property bool followLatest: true
    property bool cardMode: false
    property real contentMaximumWidth: 0
    readonly property real resolvedContentWidth: contentMaximumWidth > 0
                                                ? Math.min(width, contentMaximumWidth)
                                                : width
    signal submitRequested(string text)

    function submitComposer() {
        if (composer.text.trim().length === 0 || client.activeTurnId.length > 0) {
            return
        }
        root.followLatest = true
        root.submitRequested(composer.text)
        composer.clear()
    }

    function isUserMessage(message) {
        return message.role === "user"
    }

    function scrollToBottom() {
        Qt.callLater(function() {
            messageList.positionViewAtEnd()
        })
    }

    function messageValue(message, key, fallback) {
        var value = message[key]
        return value === undefined || value === null ? fallback : value
    }

    function selectedModelInfo() {
        for (var index = 0; index < client.models.length; index++) {
            if (client.models[index].id === client.selectedModel) {
                return client.models[index]
            }
        }
        return null
    }

    function selectedModelConfigured() {
        var model = selectedModelInfo()
        return model ? model.availability === "configured" : false
    }

    function selectedModelPrompt() {
        if (client.sessionId.length === 0) {
            return "Create a session first..."
        }
        var model = selectedModelInfo()
        if (!model) {
            return "Choose a model first..."
        }
        if (model.availability === "configured") {
            return "Tell SunCode what to do..."
        }
        if (model.provider_label) {
            return "Store a " + model.provider_label + " API key first..."
        }
        return "Store the selected provider's API key first..."
    }

    function appendMessage(message) {
        messageModel.append({
            role: messageValue(message, "role", ""),
            messageText: messageValue(message, "text", ""),
            content_sequence: messageValue(message, "content_sequence", 0),
            turn_id: messageValue(message, "turn_id", ""),
            streaming: messageValue(message, "streaming", false)
        })
    }

    function canReuseMessage(localMessage, incomingMessage) {
        var localTurn = messageValue(localMessage, "turn_id", "")
        var incomingTurn = messageValue(incomingMessage, "turn_id", "")
        var localRole = messageValue(localMessage, "role", "")
        var incomingRole = messageValue(incomingMessage, "role", "")
        if (localRole !== incomingRole) {
            return false
        }
        if (localTurn.length > 0 || incomingTurn.length > 0) {
            return localTurn === incomingTurn
        }
        return messageValue(localMessage, "content_sequence", 0) === messageValue(incomingMessage, "content_sequence", 0)
    }

    function updateMessage(index, message) {
        messageModel.set(index, {
            role: messageValue(message, "role", ""),
            messageText: messageValue(message, "text", ""),
            content_sequence: messageValue(message, "content_sequence", 0),
            turn_id: messageValue(message, "turn_id", ""),
            streaming: messageValue(message, "streaming", false)
        })
    }

    function rebuildMessages(messages) {
        messageModel.clear()
        for (var index = 0; index < messages.length; index++) {
            appendMessage(messages[index])
        }
    }

    function syncMessages() {
        var source = client.messages || []
        if (messageModel.count === 0) {
            rebuildMessages(source)
            return
        }
        if (source.length < messageModel.count) {
            rebuildMessages(source)
            return
        }
        var shared = Math.min(source.length, messageModel.count)
        for (var index = 0; index < shared; index++) {
            var localMessage = messageModel.get(index)
            var incomingMessage = source[index]
            if (!canReuseMessage(localMessage, incomingMessage)) {
                rebuildMessages(source)
                return
            }
            if (messageValue(localMessage, "messageText", "") !== messageValue(incomingMessage, "text", "")
                    || messageValue(localMessage, "streaming", false) !== messageValue(incomingMessage, "streaming", false)
                    || messageValue(localMessage, "content_sequence", 0) !== messageValue(incomingMessage, "content_sequence", 0)) {
                updateMessage(index, incomingMessage)
            }
        }
        for (var appendIndex = messageModel.count; appendIndex < source.length; appendIndex++) {
            appendMessage(source[appendIndex])
        }
    }

    Layout.fillWidth: true
    Layout.fillHeight: true
    Rectangle {
        anchors.fill: parent
        color: root.cardMode ? "transparent" : theme.workspace
        z: -1  // 放在最底层
    }

    ListModel {
        id: messageModel
    }

    ListView {
        id: messageList
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        anchors.horizontalCenter: parent.horizontalCenter
        width: root.resolvedContentWidth
        clip: true
        model: messageModel
        spacing: 0
        topMargin: 18
        bottomMargin: 0
        leftMargin: 12
        rightMargin: 24
        footer: Item {
            width: messageList.width
            height: composerDock.height + 6
        }
        boundsBehavior: Flickable.StopAtBounds

        function isNearBottom() {
            return contentY + height >= contentHeight - 6
        }

        onMovementEnded: {
            root.followLatest = isNearBottom()
        }

        onContentHeightChanged: {
            if (root.pendingInitialScroll && client.sessionId.length > 0) {
                root.pendingInitialScroll = false
                root.scrollToBottom()
                return
            }
            if (root.followLatest || client.activeTurnId.length > 0) {
                root.scrollToBottom()
            }
        }

        delegate: Item {
            required property string role
            required property string messageText
            required property int content_sequence
            required property string turn_id
            required property bool streaming
            property bool userSide: role === "user"
            width: messageList.width
            height: bubble.implicitHeight + 18

            Row {
                id: messageRow
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.leftMargin: 12
                anchors.rightMargin: userSide ? 24 : 12
                anchors.top: parent.top
                anchors.topMargin: 8
                anchors.bottom: parent.bottom
                anchors.bottomMargin: 8
                spacing: 0

                Item {
                    width: userSide ? Math.max(0, messageRow.width - bubble.width) : 0
                    height: 1
                }

                Rectangle {
                    id: bubble
                    width: Math.min(parent.width * 0.8, textBody.implicitWidth + 36)
                    implicitHeight: textBody.implicitHeight + (userSide ? 28 : (copyButton.height > 0 ? 42 : 12))
                    height: implicitHeight
                    color: userSide
                        ? (theme.isLight ? theme.surfaceActive : theme.surfaceHover)
                        : "transparent"
                    radius: theme.radiusLarge
                    border.width: userSide ? 1 : 0
                    border.color: userSide ? (theme.isLight ? theme.borderStrong : theme.border) : "transparent"

                    Column {
                        anchors.fill: parent
                        anchors.margins: userSide ? 12 : 6
                        spacing: userSide ? 0 : 6

                        Text {
                            id: textBody
                            width: bubble.width - (userSide ? 28 : 12)
                            text: messageText
                            color: theme.text
                            wrapMode: Text.Wrap
                            textFormat: streaming ? Text.PlainText : Text.MarkdownText
                            font.pixelSize: theme.typeBody
                            lineHeight: 1.25
                            onLinkActivated: Qt.openUrlExternally(link)
                        }

                        Button {
                            id: copyButton
                            width: 26
                            height: visible ? 26 : 0
                            visible: !userSide && messageText.trim().length > 0
                            hoverEnabled: true
                            property bool copied: false
                            Accessible.name: copied ? "Copied response" : "Copy response"
                            ToolTip.visible: hovered || copied
                            ToolTip.text: copied ? "Copied" : "Copy response"
                            onClicked: {
                                client.copyText(messageText)
                                copied = true
                                copyFeedback.restart()
                            }
                            background: Rectangle {
                                radius: theme.radiusSmall
                                color: copyButton.hovered ? theme.surfaceHover : "transparent"
                                border.width: copyButton.visualFocus ? 2 : 0
                                border.color: theme.accent
                            }
                            contentItem: ThemeIcon {
                                anchors.centerIn: parent
                                width: 16
                                height: 16
                                source: "qrc:/assets/icons/copy.svg"
                                color: copyButton.copied ? theme.success : theme.textMuted
                            }
                            Timer {
                                id: copyFeedback
                                interval: 1400
                                onTriggered: {
                                    copyButton.copied = false
                                }
                            }
                        }
                    }
                }

                Item {
                    width: userSide ? 0 : Math.max(0, messageRow.width - bubble.width)
                    height: 1
                }
            }
        }
    }

    Connections {
        target: client
        function onSessionIdChanged() {
            root.pendingInitialScroll = client.sessionId.length > 0
            root.followLatest = true
            messageModel.clear()
        }
        function onMessagesChanged() {
            root.syncMessages()
            if (root.pendingInitialScroll && client.sessionId.length > 0) {
                root.scrollToBottom()
                return
            }
            if (root.followLatest || client.activeTurnId.length > 0) {
                root.scrollToBottom()
            }
        }
    }

    Component.onCompleted: root.syncMessages()

    Rectangle {
        id: composerDock
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        width: root.resolvedContentWidth
        height: 120
        color: "transparent"
        z: 2

        Rectangle {
            id: composerCard
            anchors.fill: parent
            anchors.leftMargin: 24
            anchors.rightMargin: 24
            anchors.topMargin: 8
            anchors.bottomMargin: 10
            color: theme.composer
            radius: theme.radiusComposer
            border.width: composer.activeFocus ? 2 : 1
            border.color: composer.activeFocus ? theme.accentBorder : theme.borderStrong
            layer.enabled: true
            layer.effect: MultiEffect {
                shadowEnabled: true
                shadowColor: "#000000"
                shadowOpacity: 0.34
                shadowBlur: 0.65
                shadowHorizontalOffset: 0
                shadowVerticalOffset: 6
            }

            TextArea {
                id: composer
                anchors.left: parent.left
                anchors.right: parent.right
                anchors.top: parent.top
                anchors.bottom: composerActions.top
                anchors.leftMargin: 16
                anchors.rightMargin: 16
                anchors.topMargin: 14
                anchors.bottomMargin: 6
                placeholderText: selectedModelPrompt()
                color: theme.text
                placeholderTextColor: theme.textMuted
                selectionColor: theme.accent
                selectedTextColor: theme.accentInk
                font.pixelSize: theme.typeBody
                wrapMode: TextArea.Wrap
                selectByMouse: true
                enabled: client.connectionState === "connected" && client.sessionId.length > 0 && selectedModelConfigured() && client.activeTurnId.length === 0
                background: null
                padding: 0
                Keys.onPressed: function(event) {
                    if ((event.key === Qt.Key_Return || event.key === Qt.Key_Enter) && !(event.modifiers & Qt.ShiftModifier)) {
                        event.accepted = true
                        root.submitComposer()
                    }
                }
            }

            RowLayout {
                id: composerActions
                anchors.right: parent.right
                anchors.bottom: parent.bottom
                anchors.rightMargin: 12
                anchors.bottomMargin: 10
                spacing: 8

                AppComboBox {
                    id: turnModel
                    theme: root.theme
                    Layout.preferredWidth: 178
                    Layout.preferredHeight: 34
                    model: client.models
                    textRole: "id"
                    currentIndex: {
                        for (var index = 0; index < client.models.length; index++) {
                            if (client.models[index].id === client.selectedModel) return index
                        }
                        return 0
                    }
                    enabled: composer.enabled && client.activeTurnId.length === 0
                    font.family: theme.fontMono
                    font.pixelSize: theme.typeLabel
                    flat: true
                    onActivated: client.selectedModel = currentText
                    ToolTip.visible: hovered
                    ToolTip.text: "Model for this turn"
                }

                Button {
                    id: turnAction
                    Layout.preferredWidth: 38
                    Layout.preferredHeight: 38
                    hoverEnabled: true
                    enabled: client.activeTurnId.length > 0 || (composer.text.trim().length > 0 && client.connectionState === "connected" && client.sessionId.length > 0 && selectedModelConfigured())
                    Accessible.name: client.activeTurnId.length > 0 ? "Stop turn" : "Send message"
                    ToolTip.visible: hovered
                    ToolTip.text: client.activeTurnId.length > 0 ? "Stop turn" : "Send message"
                    onClicked: {
                        if (client.activeTurnId.length > 0) {
                            client.cancelTurn()
                        } else {
                            root.submitComposer()
                        }
                    }

                    background: Rectangle {
                        radius: width / 2
                        color: {
                            if (!turnAction.enabled) return theme.surfaceActive
                            if (client.activeTurnId.length > 0) return turnAction.down ? theme.dangerSurface : (turnAction.hovered ? theme.dangerBorder : theme.dangerSurface)
                            return turnAction.down ? theme.accentPressed : (turnAction.hovered ? theme.accentHover : theme.accent)
                        }
                        border.width: turnAction.visualFocus ? 2 : 0
                        border.color: theme.accent
                    }

                    contentItem: Item {
                        Rectangle {
                            visible: client.activeTurnId.length > 0
                            anchors.centerIn: parent
                            width: 12
                            height: 12
                            radius: 2
                            color: turnAction.enabled ? theme.danger : theme.textDisabled
                        }
                        ThemeIcon {
                            visible: client.activeTurnId.length === 0
                            anchors.centerIn: parent
                            width: 18
                            height: 18
                            source: "qrc:/assets/icons/arrow-up.svg"
                            color: theme.accentInk
                            opacity: turnAction.enabled ? 1 : 0.45
                        }
                    }
                }
            }
        }
    }
}
