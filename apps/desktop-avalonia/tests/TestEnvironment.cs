using System.Globalization;
using System.Runtime.CompilerServices;

namespace SunCode.Desktop.Tests;

internal static class TestEnvironment
{
    [ModuleInitializer]
    internal static void Initialize()
    {
        var directory = Path.Combine(
            Path.GetTempPath(),
            "suncode-desktop-tests",
            Environment.ProcessId.ToString(CultureInfo.InvariantCulture));
        Environment.SetEnvironmentVariable("SUNCODE_DATA_DIRECTORY", directory);
    }
}
