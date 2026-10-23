using Avalonia.Controls;
using Avalonia.VisualTree;
using AvaloniaEdit;

namespace SunCode.Desktop.Infrastructure;

internal enum DesktopEditAction { Undo, Redo, Cut, Copy, Paste, SelectAll }

// Resolve the actual focused control; menu commands never alter the composer
// merely because another, read-only surface does not support the operation.
internal static class DesktopEditCommands
{
    private static object? Target(Window? owner)
    {
        var focused = owner?.FocusManager?.GetFocusedElement() as Avalonia.Visual;
        return focused?.GetSelfAndVisualAncestors().FirstOrDefault(item => item is TextBox or SelectableTextBlock or TextEditor);
    }

    internal static bool CanExecute(Window? owner, DesktopEditAction action) => CanExecuteTarget(Target(owner), action);

    internal static bool CanExecuteTarget(object? target, DesktopEditAction action) => target switch
    {
        TextBox text when text.IsEffectivelyEnabled => action switch
        {
            DesktopEditAction.Undo => !text.IsReadOnly && text.CanUndo,
            DesktopEditAction.Redo => !text.IsReadOnly && text.CanRedo,
            DesktopEditAction.Cut => !text.IsReadOnly && text.CanCut,
            DesktopEditAction.Copy => text.CanCopy,
            DesktopEditAction.Paste => !text.IsReadOnly && text.CanPaste,
            DesktopEditAction.SelectAll => !string.IsNullOrEmpty(text.Text),
            _ => false
        },
        SelectableTextBlock text => action == DesktopEditAction.Copy ? text.CanCopy
            : action == DesktopEditAction.SelectAll && !string.IsNullOrEmpty(text.Text),
        TextEditor editor => action == DesktopEditAction.Copy ? editor.SelectionLength > 0
            : action == DesktopEditAction.SelectAll && editor.Document?.TextLength > 0,
        _ => false
    };

    internal static void Execute(Window? owner, DesktopEditAction action)
    {
        var target = Target(owner);
        if (!CanExecuteTarget(target, action)) return;
        switch (target)
        {
            case TextBox text:
                switch (action)
                {
                    case DesktopEditAction.Undo: text.Undo(); break;
                    case DesktopEditAction.Redo: text.Redo(); break;
                    case DesktopEditAction.Cut: text.Cut(); break;
                    case DesktopEditAction.Copy: text.Copy(); break;
                    case DesktopEditAction.Paste: text.Paste(); break;
                    case DesktopEditAction.SelectAll: text.SelectAll(); break;
                }
                break;
            case SelectableTextBlock text:
                if (action == DesktopEditAction.Copy) text.Copy();
                else if (action == DesktopEditAction.SelectAll) text.SelectAll();
                break;
            case TextEditor editor:
                if (action == DesktopEditAction.Copy) editor.Copy();
                else if (action == DesktopEditAction.SelectAll) editor.SelectAll();
                break;
        }
    }
}
