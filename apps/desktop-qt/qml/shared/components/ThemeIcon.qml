import QtQuick
import QtQuick.Effects

Item {
    id: root

    property url source
    property color color: "white"

    implicitWidth: 16
    implicitHeight: 16

    Image {
        id: iconSource
        anchors.fill: parent
        visible: false
        source: root.source
        sourceSize.width: Math.max(1, Math.ceil(root.width))
        sourceSize.height: Math.max(1, Math.ceil(root.height))
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
