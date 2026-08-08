import QtQuick
import QtQuick.Controls

TextField {
    id: control
    property var theme

    implicitHeight: theme.controlHeight
    color: theme.text
    placeholderTextColor: theme.textMuted
    selectionColor: theme.accent
    selectedTextColor: theme.accentInk
    font.pixelSize: theme.typeBody
    leftPadding: 11
    rightPadding: 11
    selectByMouse: true

    background: Rectangle {
        radius: theme.radiusSmall
        color: control.activeFocus ? theme.fieldFocus : theme.field
        border.width: control.activeFocus ? 2 : 1
        border.color: control.activeFocus ? theme.accent : theme.border
    }
}
