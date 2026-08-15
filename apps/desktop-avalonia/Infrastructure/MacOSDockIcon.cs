using System.Runtime.InteropServices;
using Avalonia.Controls;

namespace SunCode.Desktop;

internal static class MacOSDockIcon
{
    internal static void Apply()
    {
        if (!OperatingSystem.IsMacOS()) return;
        var path = Path.Combine(AppContext.BaseDirectory, "suncode-logo-1024.png");
        if (!File.Exists(path)) return;

        var application = Send(GetClass("NSApplication"), Selector("sharedApplication"));
        var image = Send(GetClass("NSImage"), Selector("alloc"));
        var nsPath = SendUtf8(GetClass("NSString"), Selector("stringWithUTF8String:"), path);
        image = Send(image, Selector("initWithContentsOfFile:"), nsPath);
        if (application != 0 && image != 0)
            SendVoid(application, Selector("setApplicationIconImage:"), image);

        var defaults = Send(GetClass("NSUserDefaults"), Selector("standardUserDefaults"));
        var safeAreaKey = SendUtf8(GetClass("NSString"), Selector("stringWithUTF8String:"), "NSPrefersDisplaySafeAreaCompatibilityMode");
        if (defaults != 0 && safeAreaKey != 0)
            SendBoolForKey(defaults, Selector("setBool:forKey:"), false, safeAreaKey);
    }

    internal static bool ToggleNativeFullScreen(Window window)
    {
        if (!OperatingSystem.IsMacOS() || window.TryGetPlatformHandle() is not { Handle: not 0 } handle)
            return false;

        var nativeWindow = handle.Handle;
        if (handle.HandleDescriptor?.Contains("NSView", StringComparison.OrdinalIgnoreCase) == true)
            nativeWindow = Send(nativeWindow, Selector("window"));
        if (nativeWindow == 0) return false;

        const nuint fullScreenPrimary = 1 << 7;
        var behavior = SendUIntResult(nativeWindow, Selector("collectionBehavior"));
        SendUInt(nativeWindow, Selector("setCollectionBehavior:"), behavior | fullScreenPrimary);
        const nuint fullSizeContentView = 1 << 15;
        var styleMask = SendUIntResult(nativeWindow, Selector("styleMask"));
        SendUInt(nativeWindow, Selector("setStyleMask:"), styleMask | fullSizeContentView);
        SendUInt(nativeWindow, Selector("setTitleVisibility:"), 1);
        SendUInt(nativeWindow, Selector("setTitlebarAppearsTransparent:"), 1);
        SendVoid(nativeWindow, Selector("toggleFullScreen:"), IntPtr.Zero);
        return true;
    }

    private static IntPtr GetClass(string name) => objc_getClass(name);
    private static IntPtr Selector(string name) => sel_registerName(name);

    [DllImport("/usr/lib/libobjc.A.dylib", EntryPoint = "objc_getClass", CharSet = CharSet.Ansi)]
    private static extern IntPtr objc_getClass(string name);

    [DllImport("/usr/lib/libobjc.A.dylib", EntryPoint = "sel_registerName", CharSet = CharSet.Ansi)]
    private static extern IntPtr sel_registerName(string name);

    [DllImport("/usr/lib/libobjc.A.dylib", EntryPoint = "objc_msgSend")]
    private static extern IntPtr Send(IntPtr receiver, IntPtr selector);

    [DllImport("/usr/lib/libobjc.A.dylib", EntryPoint = "objc_msgSend")]
    private static extern IntPtr Send(IntPtr receiver, IntPtr selector, IntPtr argument);

    [DllImport("/usr/lib/libobjc.A.dylib", EntryPoint = "objc_msgSend", CharSet = CharSet.Ansi)]
    private static extern IntPtr SendUtf8(IntPtr receiver, IntPtr selector, string argument);

    [DllImport("/usr/lib/libobjc.A.dylib", EntryPoint = "objc_msgSend")]
    private static extern void SendVoid(IntPtr receiver, IntPtr selector, IntPtr argument);

    [DllImport("/usr/lib/libobjc.A.dylib", EntryPoint = "objc_msgSend")]
    private static extern void SendUInt(IntPtr receiver, IntPtr selector, nuint argument);

    [DllImport("/usr/lib/libobjc.A.dylib", EntryPoint = "objc_msgSend")]
    private static extern nuint SendUIntResult(IntPtr receiver, IntPtr selector);

    [DllImport("/usr/lib/libobjc.A.dylib", EntryPoint = "objc_msgSend")]
    private static extern void SendBoolForKey(
        IntPtr receiver,
        IntPtr selector,
        [MarshalAs(UnmanagedType.I1)] bool value,
        IntPtr key);
}
