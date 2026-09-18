using Avalonia;
using Avalonia.Controls;
using Avalonia.Interactivity;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.Views.Settings.Controls;

public sealed partial class AgentSettingsControl : UserControl
{
    public static readonly StyledProperty<IEnumerable<AgentItem>?> AgentsProperty =
        AvaloniaProperty.Register<AgentSettingsControl, IEnumerable<AgentItem>?>(nameof(Agents));
    public static readonly StyledProperty<string?> SelectedAgentIdProperty =
        AvaloniaProperty.Register<AgentSettingsControl, string?>(nameof(SelectedAgentId));

    public event EventHandler<string>? AgentSelected;

    public AgentSettingsControl()
    {
        InitializeComponent();
        SyncView();
    }

    public IEnumerable<AgentItem>? Agents { get => GetValue(AgentsProperty); set => SetValue(AgentsProperty, value); }
    public string? SelectedAgentId { get => GetValue(SelectedAgentIdProperty); set => SetValue(SelectedAgentIdProperty, value); }

    protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
    {
        base.OnPropertyChanged(change);
        if (change.Property == AgentsProperty || change.Property == SelectedAgentIdProperty) SyncView();
    }

    private void SelectAgent(object? sender, RoutedEventArgs e)
    {
        if (sender is Button { CommandParameter: AgentItem agent }) AgentSelected?.Invoke(this, agent.Id);
    }

    private void SyncView()
    {
        if (OverviewPanel is null || DetailPanel is null) return;
        AgentItemsControl.ItemsSource = Agents;
        var selected = Agents?.FirstOrDefault(agent => agent.Id == SelectedAgentId);
        OverviewPanel.IsVisible = selected is null;
        DetailPanel.IsVisible = selected is not null;
        if (selected is null) return;
        DetailTitle.Text = selected.DisplayName;
        DetailDescription.Text = selected.Description;
        DetailDisplayName.Text = selected.DisplayName;
        DetailName.Text = selected.Name;
        DetailId.Text = selected.Id;
        DetailVersion.Text = selected.Version.ToString();
        DetailModelPolicy.Text = selected.ModelPolicy;
        DetailToolLimit.Text = selected.ToolLimitText;
        AllowedTools.ItemsSource = selected.AllowedTools;
        DetailMcp.Text = selected.McpPolicy;
        DetailDelegate.Text = selected.DelegateText;
    }
}
