import QtQuick
import QtQuick.Controls

Label {
    property var theme
    color: theme.textSecondary
    font.family: theme.fontUi
    font.pixelSize: theme.typeLabel
    font.weight: Font.DemiBold
}
