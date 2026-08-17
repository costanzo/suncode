using System.Runtime.InteropServices;

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
    private static extern void SendBoolForKey(
        IntPtr receiver,
        IntPtr selector,
        [MarshalAs(UnmanagedType.I1)] bool value,
        IntPtr key);
}
