import QtQuick
import QtQuick.Window

Item {
    id: root

    property var controller
    property int handleSize: 6
    property bool topEdgeEnabled: true
    property bool bottomEdgeEnabled: true
    property bool active: Window.window && Window.window.visibility !== Window.FullScreen

    function startResize(edges, mouseArea, mouse) {
        if (active && controller) {
            controller.startResize(mouseArea, mouse.x, mouse.y, edges)
        }
    }

    MouseArea {
        id: topHandle
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        height: root.handleSize
        enabled: root.active && root.topEdgeEnabled
        acceptedButtons: Qt.LeftButton
        cursorShape: Qt.SizeVerCursor
        onPressed: function(mouse) { root.startResize(Qt.TopEdge, topHandle, mouse) }
    }

    MouseArea {
        id: bottomHandle
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        height: root.handleSize
        enabled: root.active && root.bottomEdgeEnabled
        acceptedButtons: Qt.LeftButton
        cursorShape: Qt.SizeVerCursor
        onPressed: function(mouse) { root.startResize(Qt.BottomEdge, bottomHandle, mouse) }
    }

    MouseArea {
        id: leftHandle
        anchors.left: parent.left
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: root.handleSize
        enabled: root.active
        acceptedButtons: Qt.LeftButton
        cursorShape: Qt.SizeHorCursor
        onPressed: function(mouse) { root.startResize(Qt.LeftEdge, leftHandle, mouse) }
    }

    MouseArea {
        id: rightHandle
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.bottom: parent.bottom
        width: root.handleSize
        enabled: root.active
        acceptedButtons: Qt.LeftButton
        cursorShape: Qt.SizeHorCursor
        onPressed: function(mouse) { root.startResize(Qt.RightEdge, rightHandle, mouse) }
    }

    MouseArea {
        id: topLeftHandle
        anchors.left: parent.left
        anchors.top: parent.top
        width: root.handleSize
        height: root.handleSize
        enabled: root.active && root.topEdgeEnabled
        acceptedButtons: Qt.LeftButton
        cursorShape: Qt.SizeFDiagCursor
        onPressed: function(mouse) { root.startResize(Qt.LeftEdge | Qt.TopEdge, topLeftHandle, mouse) }
    }

    MouseArea {
        id: topRightHandle
        anchors.right: parent.right
        anchors.top: parent.top
        width: root.handleSize
        height: root.handleSize
        enabled: root.active && root.topEdgeEnabled
        acceptedButtons: Qt.LeftButton
        cursorShape: Qt.SizeBDiagCursor
        onPressed: function(mouse) { root.startResize(Qt.RightEdge | Qt.TopEdge, topRightHandle, mouse) }
    }

    MouseArea {
        id: bottomLeftHandle
        anchors.left: parent.left
        anchors.bottom: parent.bottom
        width: root.handleSize
        height: root.handleSize
        enabled: root.active && root.bottomEdgeEnabled
        acceptedButtons: Qt.LeftButton
        cursorShape: Qt.SizeBDiagCursor
        onPressed: function(mouse) { root.startResize(Qt.LeftEdge | Qt.BottomEdge, bottomLeftHandle, mouse) }
    }

    MouseArea {
        id: bottomRightHandle
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        width: root.handleSize
        height: root.handleSize
        enabled: root.active && root.bottomEdgeEnabled
        acceptedButtons: Qt.LeftButton
        cursorShape: Qt.SizeFDiagCursor
        onPressed: function(mouse) { root.startResize(Qt.RightEdge | Qt.BottomEdge, bottomRightHandle, mouse) }
    }
}
