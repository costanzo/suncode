using System.Collections.ObjectModel;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.Models;

namespace SunCode.Desktop.ViewModels;

// Turn-grouped tool calls for the selected session and the inspector
// selection. The conversation owns event decoding and the compact active-tool
// row; drawer visibility stays with the window layout.
public sealed class ToolActivityViewModel : ObservableObject
{
    private ToolActivityTurnItem? _selectedTurn;
    private ToolActivityItem? _selectedTool;

    public ObservableCollection<ToolActivityTurnItem> Turns { get; } = [];

    public ToolActivityTurnItem? SelectedTurn
    {
        get => _selectedTurn;
        private set
        {
            if (SetProperty(ref _selectedTurn, value))
                OnPropertyChanged(nameof(SelectedTitle));
        }
    }

    public ToolActivityItem? SelectedTool
    {
        get => _selectedTool;
        private set
        {
            if (SetProperty(ref _selectedTool, value))
            {
                OnPropertyChanged(nameof(HasSelectedTool));
                OnPropertyChanged(nameof(SelectedTitle));
            }
        }
    }

    public bool HasTurns => Turns.Count > 0;
    public bool HasSelectedTool => SelectedTool is not null;
    public string Summary => $"{Turns.Count} {(Turns.Count == 1 ? "turn" : "turns")} · {Turns.Sum(turn => turn.Tools.Count)} calls";
    public string SelectedTitle => SelectedTool is null || SelectedTurn is null
        ? "Select a tool call"
        : $"{SelectedTurn.Title} · {SelectedTool.StateText}";

    // The most recent turn that is still running and has a running tool.
    internal (ToolActivityTurnItem Turn, ToolActivityItem Tool)? ActiveTool
    {
        get
        {
            var turn = Turns.LastOrDefault(item => item.IsActive);
            var tool = turn?.Tools.LastOrDefault(item => item.IsActive);
            return turn is null || tool is null ? null : (turn, tool);
        }
    }

    public void Select(ToolActivityTurnItem turn, ToolActivityItem tool)
    {
        turn.IsExpanded = true;
        SelectedTurn = turn;
        SelectedTool = tool;
    }

    public void Select(ToolActivityItem tool)
    {
        if (Turns.FirstOrDefault(item => item.TurnId == tool.TurnId) is { } turn) Select(turn, tool);
    }

    public void ToggleTurn(ToolActivityTurnItem turn)
    {
        turn.IsExpanded = !turn.IsExpanded;
        SelectedTurn = turn;
        if (SelectedTool is null || SelectedTool.TurnId != turn.TurnId)
            SelectedTool = turn.Tools.FirstOrDefault();
    }

    internal bool TrySelect(string turnId, string toolCallId)
    {
        var turn = Turns.FirstOrDefault(item => item.TurnId == turnId);
        var tool = turn?.Tools.FirstOrDefault(item => item.ToolCallId == toolCallId);
        if (turn is null || tool is null) return false;
        Select(turn, tool);
        return true;
    }

    internal ToolActivityTurnItem EnsureTurn(string turnId, string preview = "")
    {
        var existing = Turns.FirstOrDefault(item => item.TurnId == turnId);
        if (existing is not null)
        {
            if (!string.IsNullOrWhiteSpace(preview)) existing.Update(existing.State, preview);
            return existing;
        }
        var created = new ToolActivityTurnItem(turnId, Turns.Count + 1, "admitted", preview, string.Empty)
        {
            IsExpanded = true
        };
        Turns.Add(created);
        NotifyTurnsChanged();
        return created;
    }

    // Adds or updates one tool call and follows it when nothing is selected or
    // the selection is already tracking the running turn.
    internal void Upsert(
        ToolActivityTurnItem turn,
        string toolCallId,
        string name,
        string state,
        string request,
        string result,
        string output,
        string error)
    {
        var existing = turn.Tools.FirstOrDefault(item => item.ToolCallId == toolCallId);
        if (existing is null)
        {
            existing = new ToolActivityItem(turn.TurnId, toolCallId, name, state, request, result, output, error, string.Empty);
            turn.Tools.Add(existing);
            NotifyTurnsChanged();
        }
        else
        {
            existing.Update(name, state, request, result, output, error);
        }
        if (SelectedTool is null || (SelectedTurn?.IsActive == true && existing.IsActive))
            Select(turn, existing);
    }

    internal void ReplaceAll(IEnumerable<ToolActivityTurnItem> turns)
    {
        Turns.Clear();
        foreach (var turn in turns) Turns.Add(turn);
        SelectDefault();
        NotifyTurnsChanged();
    }

    // Turn state changes the call counts shown in the summary.
    internal void NotifyTurnsChanged()
    {
        OnPropertyChanged(nameof(HasTurns));
        OnPropertyChanged(nameof(Summary));
    }

    private void SelectDefault()
    {
        var turn = Turns.LastOrDefault(item => item.IsActive && item.Tools.Any(tool => tool.IsActive))
            ?? Turns.LastOrDefault(item => item.Tools.Count > 0);
        var tool = turn?.Tools.LastOrDefault(item => item.IsActive) ?? turn?.Tools.FirstOrDefault();
        if (turn is null || tool is null)
        {
            SelectedTurn = null;
            SelectedTool = null;
            return;
        }
        Select(turn, tool);
    }
}
