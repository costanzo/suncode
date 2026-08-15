cmake_minimum_required(VERSION 3.21)

project(suncode-desktop VERSION 0.1.0 LANGUAGES CXX)

include(GNUInstallDirs)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTOMOC_PATH_PREFIX ON)

find_package(Qt6 6.5 REQUIRED COMPONENTS Quick QuickControls2 Concurrent)
find_package(ZLIB REQUIRED)
if(APPLE)
    find_package(Iconv REQUIRED)
endif()

find_program(CARGO_EXECUTABLE cargo REQUIRED)

if(APPLE)
    set(SUNCODE_APP_ICON "${CMAKE_CURRENT_SOURCE_DIR}/assets/suncode-desktop.icns")
    set_source_files_properties("${SUNCODE_APP_ICON}" PROPERTIES MACOSX_PACKAGE_LOCATION "Resources")
endif()

set(SUNCODE_RUNTIME_DIR "${CMAKE_CURRENT_SOURCE_DIR}/../../runtime")
set(SUNCODE_RUNTIME_TARGET_DIR "${SUNCODE_RUNTIME_DIR}/target")
if(WIN32)
    set(SUNCODE_RUNTIME_SDK_LIB "${SUNCODE_RUNTIME_TARGET_DIR}/debug/suncode_runtime.lib")
else()
    set(SUNCODE_RUNTIME_SDK_LIB "${SUNCODE_RUNTIME_TARGET_DIR}/debug/libsuncode_runtime.a")
endif()
file(GLOB_RECURSE SUNCODE_RUNTIME_SOURCES CONFIGURE_DEPENDS
    "${SUNCODE_RUNTIME_DIR}/Cargo.toml"
    "${SUNCODE_RUNTIME_DIR}/Cargo.lock"
    "${SUNCODE_RUNTIME_DIR}/crates/**/*.rs"
    "${SUNCODE_RUNTIME_DIR}/crates/**/*.sql"
)
add_custom_command(
    OUTPUT "${SUNCODE_RUNTIME_SDK_LIB}"
    COMMAND "${CMAKE_COMMAND}" -E env
            "CARGO_TARGET_DIR=${SUNCODE_RUNTIME_TARGET_DIR}"
            "${CARGO_EXECUTABLE}" build --manifest-path "${SUNCODE_RUNTIME_DIR}/Cargo.toml" -p suncode-runtime --lib
    WORKING_DIRECTORY "${SUNCODE_RUNTIME_DIR}"
    DEPENDS ${SUNCODE_RUNTIME_SOURCES}
    VERBATIM
)
add_custom_target(suncode-runtime-sdk DEPENDS "${SUNCODE_RUNTIME_SDK_LIB}")

qt_standard_project_setup(REQUIRES 6.5)
qt_policy(SET QTP0004 NEW)

qt_add_executable(suncode-desktop
    MACOSX_BUNDLE
    src/main.cpp
    src/runtimeclient.cpp
    src/runtimeclient.h
    src/runtime_sdk.h
    src/runtimeclient_support.cpp
    src/runtimeclient_support.h
    ${SUNCODE_APP_ICON}
)

if(APPLE)
    set_target_properties(suncode-desktop PROPERTIES
        MACOSX_BUNDLE_ICON_FILE "suncode-desktop.icns"
    )
endif()

qt_add_resources(suncode-desktop suncode-assets
    PREFIX "/assets"
    BASE "assets"
    FILES
        assets/logo/suncode-logo-1024.png
        assets/logo/suncode-logo-256.png
        assets/logo/suncode-logo-small-256.png
        assets/logo/suncode-logo-small-64.png
        assets/logo/suncode-logo-small-32.png
        assets/icons/arrow-up.svg
        assets/icons/chevron-right.svg
        assets/icons/close.svg
        assets/icons/copy.svg
        assets/icons/git-branch.svg
        assets/icons/lock.svg
        assets/icons/maximize.svg
        assets/icons/minus.svg
        assets/icons/more-horizontal.svg
        assets/icons/panel-left.svg
        assets/icons/panel-right.svg
        assets/icons/plus.svg
        assets/icons/restore.svg
        assets/icons/refresh.svg
        assets/icons/sidebar-project.svg
        assets/icons/sidebar-review.svg
        assets/icons/settings.svg
        assets/icons/unlock.svg
        assets/icons/windows-close.svg
        assets/icons/windows-maximize.svg
        assets/icons/windows-minimize.svg
        assets/traffic-lights/1-close-1-normal.svg
        assets/traffic-lights/2-close-2-hover.svg
        assets/traffic-lights/2-close-3-press.svg
        assets/traffic-lights/2-minimize-1-normal.svg
        assets/traffic-lights/2-minimize-2-hover.svg
        assets/traffic-lights/2-minimize-3-press.svg
        assets/traffic-lights/3-maximize-1-normal.svg
        assets/traffic-lights/3-maximize-2-hover.svg
        assets/traffic-lights/3-maximize-3-press.svg
)

qt_add_qml_module(suncode-desktop
    URI SunCode.Desktop
    VERSION 1.0
    QML_FILES
        # Application shells and entry points.
        qml/app/Main.qml
        qml/app/ProjectHub.qml
        qml/app/ProjectWindow.qml
        # Product feature modules.
        qml/features/project/ConnectionPanel.qml
        qml/features/git/GitDrawer.qml
        qml/features/conversation/ConversationPanel.qml
        qml/features/review/ReviewPanel.qml
        qml/features/review/AgentProcessPanel.qml
        qml/features/settings/GlobalSettings.qml
        # Shared presentation primitives.
        qml/shared/components/AppButton.qml
        qml/shared/components/AppComboBox.qml
        qml/shared/components/AppField.qml
        qml/shared/components/SectionLabel.qml
        qml/shared/components/ThemeIcon.qml
        qml/shared/navigation/SidebarToggleButton.qml
        qml/shared/navigation/SidebarRail.qml
        qml/shared/navigation/SidebarLockButton.qml
        qml/shared/theme/Theme.qml
        qml/shared/window/WindowStateController.qml
        qml/shared/window/WindowResizeHandles.qml
)

target_include_directories(suncode-desktop PRIVATE src)
add_dependencies(suncode-desktop suncode-runtime-sdk)
target_link_libraries(suncode-desktop PRIVATE
    Qt6::Quick
    Qt6::QuickControls2
    Qt6::Concurrent
    "${SUNCODE_RUNTIME_SDK_LIB}"
    ZLIB::ZLIB
)

if(APPLE)
    target_link_libraries(suncode-desktop PRIVATE Iconv::Iconv)
endif()

if(APPLE)
    add_custom_command(TARGET suncode-desktop POST_BUILD
        COMMAND "${CMAKE_COMMAND}" -E copy_if_different
                "$<TARGET_FILE:suncode-desktop>"
                "${CMAKE_CURRENT_BINARY_DIR}/suncode-desktop"
        COMMENT "Refreshing command-line launcher at ${CMAKE_CURRENT_BINARY_DIR}/suncode-desktop"
        VERBATIM
    )
endif()

install(TARGETS suncode-desktop
    BUNDLE DESTINATION .
    RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
)
