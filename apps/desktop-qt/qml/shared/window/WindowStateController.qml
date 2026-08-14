import QtQuick
import QtQuick.Window

Item {
    id: root

    property var window
    property rect normalGeometry: Qt.rect(0, 0, 0, 0)
    visible: false
    width: 0
    height: 0

    function rememberNormalGeometry() {
        if (!window || window.visibility === Window.Maximized || window.visibility === Window.FullScreen) {
            return
        }
        normalGeometry = Qt.rect(window.x, window.y, window.width, window.height)
    }

    function restoreNormalGeometry() {
        if (!window || normalGeometry.width <= 0 || normalGeometry.height <= 0) {
            return
        }
        window.x = Math.round(normalGeometry.x)
        window.y = Math.round(normalGeometry.y)
        window.width = Math.round(normalGeometry.width)
        window.height = Math.round(normalGeometry.height)
    }

    function toggleMaximized() {
        if (!window) {
            return
        }
        if (window.visibility === Window.Maximized) {
            window.showNormal()
            Qt.callLater(root.restoreNormalGeometry)
        } else {
            rememberNormalGeometry()
            window.showMaximized()
        }
    }

    function toggleFullScreen() {
        if (!window) {
            return
        }
        if (window.visibility === Window.FullScreen) {
            window.showNormal()
            Qt.callLater(root.restoreNormalGeometry)
        } else {
            rememberNormalGeometry()
            window.showFullScreen()
        }
    }

    function startMove(item, mouseX, mouseY) {
        if (!window || !item) {
            return
        }
        if (window.visibility !== Window.Maximized) {
            window.startSystemMove()
            return
        }

        var globalPoint = item.mapToGlobal(mouseX, mouseY)
        var restoreWidth = normalGeometry.width > 0 ? normalGeometry.width : Math.max(window.minimumWidth, Math.round(window.width * 0.72))
        var restoreHeight = normalGeometry.height > 0 ? normalGeometry.height : Math.max(window.minimumHeight, Math.round(window.height * 0.72))
        var horizontalRatio = item.width > 0 ? Math.max(0.12, Math.min(0.88, mouseX / item.width)) : 0.5

        window.showNormal()
        Qt.callLater(function() {
            window.width = restoreWidth
            window.height = restoreHeight
            window.x = Math.round(globalPoint.x - restoreWidth * horizontalRatio)
            window.y = Math.round(globalPoint.y - Math.min(mouseY, 18))
            root.rememberNormalGeometry()
            window.startSystemMove()
        })
    }

    function startResize(item, mouseX, mouseY, edges) {
        if (!window || !item || window.visibility === Window.FullScreen) {
            return
        }
        if (window.visibility !== Window.Maximized) {
            window.startSystemResize(edges)
            return
        }

        var globalPoint = item.mapToGlobal(mouseX, mouseY)
        var restoreWidth = normalGeometry.width > 0 ? normalGeometry.width : Math.max(window.minimumWidth, Math.round(window.width * 0.72))
        var restoreHeight = normalGeometry.height > 0 ? normalGeometry.height : Math.max(window.minimumHeight, Math.round(window.height * 0.72))

        window.showNormal()
        Qt.callLater(function() {
            window.width = restoreWidth
            window.height = restoreHeight

            if (edges & Qt.LeftEdge) {
                window.x = Math.round(globalPoint.x)
            } else if (edges & Qt.RightEdge) {
                window.x = Math.round(globalPoint.x - restoreWidth)
            }

            if (edges & Qt.TopEdge) {
                window.y = Math.round(globalPoint.y)
            } else if (edges & Qt.BottomEdge) {
                window.y = Math.round(globalPoint.y - restoreHeight)
            }

            root.rememberNormalGeometry()
            window.startSystemResize(edges)
        })
    }

    Component.onCompleted: rememberNormalGeometry()

    Connections {
        target: root.window
        ignoreUnknownSignals: true

        function onXChanged() { root.rememberNormalGeometry() }
        function onYChanged() { root.rememberNormalGeometry() }
        function onWidthChanged() { root.rememberNormalGeometry() }
        function onHeightChanged() { root.rememberNormalGeometry() }
        function onVisibilityChanged() { root.rememberNormalGeometry() }
    }
}
