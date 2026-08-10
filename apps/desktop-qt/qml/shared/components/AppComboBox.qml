pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls

ComboBox {
    id: control

    property var theme

    implicitHeight: theme.controlHeight
    hoverEnabled: true
    focusPolicy: Qt.TabFocus
    leftPadding: 12
    rightPadding: 34

    delegate: ItemDelegate {
        id: optionDelegate

        required property int index

        width: control.popup ? control.popup.availableWidth : control.width
        implicitHeight: 34
        hoverEnabled: true
        highlighted: control.highlightedIndex === optionDelegate.index
        Accessible.name: control.textAt(optionDelegate.index)

        contentItem: Text {
            text: control.textAt(optionDelegate.index)
            color: control.currentIndex === optionDelegate.index
                   ? control.theme.text
                   : control.theme.textSecondary
            font: control.font
            verticalAlignment: Text.AlignVCenter
            elide: Text.ElideRight
        }

        background: Rectangle {
            radius: control.theme.radiusSmall
            color: optionDelegate.highlighted || optionDelegate.hovered
                   ? control.theme.surfaceHover
                   : control.currentIndex === optionDelegate.index
                     ? control.theme.surfaceActive
                     : "transparent"
        }
    }

    indicator: ThemeIcon {
        x: control.width - width - 11
        y: Math.round((control.height - height) / 2)
        width: 14
        height: 14
        source: "qrc:/assets/icons/chevron-right.svg"
        color: control.enabled ? control.theme.textSecondary : control.theme.textDisabled
        rotation: 90
    }

    contentItem: Text {
        leftPadding: control.leftPadding
        rightPadding: control.rightPadding
        text: control.displayText
        color: control.enabled ? control.theme.text : control.theme.textDisabled
        font: control.font
        verticalAlignment: Text.AlignVCenter
        elide: Text.ElideRight
    }

    background: Rectangle {
        radius: control.theme.radiusMedium
        color: !control.enabled
               ? control.theme.surface
               : control.activeFocus
                 ? control.theme.fieldFocus
                 : control.hovered ? control.theme.surfaceHover : control.theme.field
        border.width: control.activeFocus ? 2 : 1
        border.color: control.activeFocus ? control.theme.accent : control.theme.border
    }

    popup: Popup {
        y: control.height + 4
        width: control.width
        implicitHeight: Math.min(contentItem.implicitHeight + topPadding + bottomPadding, 240)
        padding: 4
        margins: 8

        contentItem: ListView {
            clip: true
            implicitHeight: contentHeight
            model: control.popup.visible ? control.delegateModel : null
            currentIndex: control.highlightedIndex
            spacing: 2
            boundsBehavior: Flickable.StopAtBounds
        }

        background: Rectangle {
            radius: control.theme.radiusMedium
            color: control.theme.surfaceRaised
            border.width: 1
            border.color: control.theme.borderStrong
        }
    }
}
