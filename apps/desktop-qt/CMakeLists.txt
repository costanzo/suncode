cmake_minimum_required(VERSION 3.21)

project(suncode-desktop VERSION 0.1.0 LANGUAGES CXX)

include(GNUInstallDirs)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)
set(CMAKE_AUTOMOC_PATH_PREFIX ON)

find_package(Qt6 6.5 REQUIRED COMPONENTS Quick QuickControls2 Concurrent)

find_program(CARGO_EXECUTABLE cargo REQUIRED)

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
    src/main.cpp
    src/runtimeclient.cpp
    src/runtimeclient.h
    src/runtime_sdk.h
    src/runtimeclient_support.cpp
    src/runtimeclient_support.h
)

qt_add_qml_module(suncode-desktop
    URI Suncode.Desktop
    VERSION 1.0
    QML_FILES
        qml/Main.qml
        qml/ProjectHub.qml
        qml/ProjectWindow.qml
        qml/GlobalSettings.qml
        qml/Theme.qml
        qml/AppButton.qml
        qml/AppField.qml
        qml/SectionLabel.qml
        qml/SidebarToggleButton.qml
        qml/SidebarRail.qml
        qml/SidebarLockButton.qml
        qml/ConnectionPanel.qml
        qml/ConversationPanel.qml
        qml/ReviewPanel.qml
        qml/AgentProcessPanel.qml
)

target_include_directories(suncode-desktop PRIVATE src)
add_dependencies(suncode-desktop suncode-runtime-sdk)
target_link_libraries(suncode-desktop PRIVATE Qt6::Quick Qt6::QuickControls2 Qt6::Concurrent "${SUNCODE_RUNTIME_SDK_LIB}")

install(TARGETS suncode-desktop
    BUNDLE DESTINATION .
    RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
)
