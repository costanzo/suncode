namespace SunCode.Desktop.Infrastructure;

internal static class AppDataPaths
{
    public static string DataDirectory
    {
        get
        {
            var configured = Environment.GetEnvironmentVariable("SUNCODE_DATA_DIRECTORY");
            if (configured is not null) return configured;

            var userProfile = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);
            if (string.IsNullOrWhiteSpace(userProfile))
            {
                userProfile = Environment.GetEnvironmentVariable("HOME")
                    ?? Environment.GetEnvironmentVariable("USERPROFILE")
                    ?? ".";
            }

            return Path.Combine(userProfile, ".suncode");
        }
    }

    public static string DefaultLogDirectory => Path.Combine(DataDirectory, "logs");

    public static string DefaultImageDirectory => Path.Combine(DataDirectory, "data", "images");
}
