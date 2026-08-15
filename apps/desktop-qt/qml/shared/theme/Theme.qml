import QtQuick

QtObject {
    property string mode: "dark"
    readonly property bool isLight: mode === "light"

    readonly property color canvas: isLight ? "#f3f5f7" : "#0d0f12"
    readonly property color surface: isLight ? "#ffffff" : "#121519"
    readonly property color surfaceRaised: isLight ? "#f8fafc" : "#181c21"
    readonly property color surfaceHover: isLight ? "#eef2f5" : "#1e2329"
    readonly property color surfaceActive: isLight ? "#e7edf1" : "#242a31"
    readonly property color sidebar: isLight ? "#e9eef2" : "#0b0e12"
    readonly property color inspector: isLight ? "#edf1f4" : "#0c0f13"
    readonly property color workspace: isLight ? "#f8fafb" : "#13171c"
    readonly property color composer: isLight ? "#ffffff" : "#181d23"
    readonly property color field: isLight ? "#f4f7f9" : "#15191e"
    readonly property color fieldFocus: isLight ? "#ffffff" : "#191f24"
    readonly property color border: isLight ? "#d5dde4" : "#292f36"
    readonly property color borderStrong: isLight ? "#bac6d0" : "#46505b"

    readonly property color text: isLight ? "#17202a" : "#edf0f3"
    readonly property color textSecondary: isLight ? "#44515e" : "#a7afb9"
    readonly property color textMuted: isLight ? "#778491" : "#727c87"
    readonly property color textDisabled: isLight ? "#a6b0ba" : "#525a64"

    readonly property color accent: isLight ? "#2f8f7d" : "#69c5b0"
    readonly property color accentHover: isLight ? "#277f6f" : "#7dd2be"
    readonly property color accentPressed: isLight ? "#226f61" : "#55ae9a"
    readonly property color accentInk: isLight ? "#ffffff" : "#07120f"
    readonly property color accentSurface: isLight ? "#e0f2ee" : "#172824"
    readonly property color accentBorder: isLight ? "#93cfc2" : "#315e54"

    readonly property color success: isLight ? "#27845b" : "#78c99b"
    readonly property color successSurface: isLight ? "#e4f3ea" : "#17251e"
    readonly property color warning: isLight ? "#9a6a20" : "#ddb16c"
    readonly property color warningSurface: isLight ? "#fff2dc" : "#2a2115"
    readonly property color warningBorder: isLight ? "#dfbe83" : "#5d492d"
    readonly property color danger: isLight ? "#b6463f" : "#e68a83"
    readonly property color dangerSurface: isLight ? "#fae6e4" : "#2a1919"
    readonly property color dangerBorder: isLight ? "#dda09b" : "#633735"

    readonly property int radiusSmall: 6
    readonly property int radiusMedium: 10
    readonly property int radiusLarge: 14
    readonly property int radiusComposer: 16
    readonly property int workspacePadding: 10
    readonly property int workspaceGap: 8
    readonly property int controlHeight: 36
    readonly property int compactControlHeight: 30
    readonly property int panelPadding: 16
    readonly property int sectionGap: 24

    readonly property string fontUi: typeof SunCodeUiFontFamily === "undefined" ? "Noto Sans" : SunCodeUiFontFamily
    readonly property string fontCjk: typeof SunCodeCjkFontFamily === "undefined" ? "Noto Sans CJK SC" : SunCodeCjkFontFamily
    readonly property string fontMono: typeof SunCodeMonoFontFamily === "undefined" ? "JetBrains Mono" : SunCodeMonoFontFamily

    readonly property int typeCaption: 11
    readonly property int typeLabel: 12
    readonly property int typeBody: 14
    readonly property int typeTitle: 16
    readonly property int typeHeading: 20
}
