import QtQuick
import QtQuick.Effects
import QtQuick.Window

Item {
    id: root

    property url source
    property color color: "white"
    readonly property real effectiveDevicePixelRatio: root.Window.window
                                                     ? root.Window.window.devicePixelRatio
                                                     : 1

    implicitWidth: 16
    implicitHeight: 16

    Image {
        id: iconSource
        anchors.fill: parent
        visible: false
        source: root.source
        sourceSize.width: Math.max(1, Math.ceil(root.width * root.effectiveDevicePixelRatio))
        sourceSize.height: Math.max(1, Math.ceil(root.height * root.effectiveDevicePixelRatio))
        fillMode: Image.PreserveAspectFit
        smooth: true
        mipmap: true
    }

    MultiEffect {
        anchors.fill: parent
        source: iconSource
        autoPaddingEnabled: false
        colorization: 1
        colorizationColor: root.color
    }
}
