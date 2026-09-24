using Avalonia;
using Avalonia.Controls;

namespace SunCode.Desktop.Controls;

public sealed partial class SCStatusDot : UserControl
{
    public static readonly StyledProperty<string?> StatusProperty =
        AvaloniaProperty.Register<SCStatusDot, string?>(nameof(Status));

    private const string DefaultStatus = "idle";
    
    public SCStatusDot()
    {
        InitializeComponent();
    }
    
    public string? Status
    {
        get => GetValue(StatusProperty);
        set => SetValue(StatusProperty, value);
    }

    protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
    {
        base.OnPropertyChanged(change);
        if (change.Property == StatusProperty)
        {
            SyncState();
        }
    }

    protected override void OnInitialized()
    {
        base.OnInitialized();
        SyncState();
    }

    private void SyncState()
    {
        var status = Normalize(Status);
        if (DotGrid is not null)
        {
            DotGrid.Classes.Set("running", status == "running");
            DotGrid.Classes.Set("compacting", status == "compacting");
            DotGrid.Classes.Set("approval", status == "approval");
            DotGrid.Classes.Set("question", status == "question");
            DotGrid.Classes.Set("failed", status == "failed");
        }
    }

    private static string Normalize(string? status)
    {
        if (string.IsNullOrWhiteSpace(status)) return DefaultStatus;
        return status.Trim().ToLowerInvariant() switch
        {
            "running" => "running",
            "compacting" => "compacting",
            "approval" => "approval",
            "question" => "question",
            "failed" => "failed",
            _ => DefaultStatus
        };
    }

}