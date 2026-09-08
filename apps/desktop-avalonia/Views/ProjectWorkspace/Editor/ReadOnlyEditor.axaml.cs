using System.ComponentModel;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Media;
using Avalonia.Styling;
using AvaloniaEdit.TextMate;
using SunCode.Desktop.Infrastructure;
using SunCode.Desktop.ViewModels;
using TextMateSharp.Grammars;
using TextMateInstallation = AvaloniaEdit.TextMate.TextMate.Installation;

namespace SunCode.Desktop.Views.ProjectWorkspace.Editor;

public sealed partial class ReadOnlyEditor : UserControl
{
    private RegistryOptions? _registryOptions;
    private TextMateInstallation? _textMateInstallation;
    private DesktopViewModel? _viewModel;
    private bool _isAttached;

    public ReadOnlyEditor()
    {
        InitializeComponent();

        SourceEditor.Options.EnableTextDragDrop = false;
        SourceEditor.Options.HighlightCurrentLine = false;
        SourceEditor.TextArea.RightClickMovesCaret = false;
        SourceEditor.TextArea.Caret.CaretBrush = Brushes.Transparent;

        DataContextChanged += OnDataContextChanged;
        ActualThemeVariantChanged += OnActualThemeVariantChanged;
        AttachedToVisualTree += (_, _) =>
        {
            _isAttached = true;
            AttachViewModel();
        };
        DetachedFromVisualTree += (_, _) =>
        {
            _isAttached = false;
            DisposeTextMate();
        };
    }

    private void OnDataContextChanged(object? sender, EventArgs eventArgs)
    {
        if (!_isAttached) return;
        AttachViewModel();
    }

    private void AttachViewModel()
    {
        if (_viewModel is not null) _viewModel.PropertyChanged -= OnViewModelPropertyChanged;
        _viewModel = DataContext as DesktopViewModel;
        if (_viewModel is not null) _viewModel.PropertyChanged += OnViewModelPropertyChanged;
        SourceEditor.Text = _viewModel?.EditorContent ?? string.Empty;
        ApplySyntaxGrammar();
    }

    private void OnViewModelPropertyChanged(object? sender, PropertyChangedEventArgs eventArgs)
    {
        if (eventArgs.PropertyName is nameof(DesktopViewModel.EditorContent))
            SourceEditor.Text = _viewModel?.EditorContent ?? string.Empty;
        else if (eventArgs.PropertyName is nameof(DesktopViewModel.EditorFileName))
            ApplySyntaxGrammar();
    }

    private void OnActualThemeVariantChanged(object? sender, EventArgs eventArgs) => ApplyTextMateTheme();

    private void EnsureTextMateInstalled()
    {
        if (_textMateInstallation is not null) return;

        _registryOptions = new RegistryOptions(CurrentThemeName());
        _textMateInstallation = SourceEditor.InstallTextMate(
            _registryOptions,
            exceptionHandler: exception =>
                DiagnosticLog.Error("editor.syntax", $"TextMate failure type={exception.GetType().Name}"));
        _textMateInstallation.AppliedTheme += OnTextMateThemeApplied;
        ApplyEditorThemeColors(_textMateInstallation);
    }

    private void ApplySyntaxGrammar()
    {
        if (_viewModel is null || string.IsNullOrEmpty(_viewModel.EditorFileName)) return;

        EnsureTextMateInstalled();
        var grammarExtension = EditorLanguage.FromFileName(_viewModel.EditorFileName).GrammarExtension;
        var scope = string.IsNullOrEmpty(grammarExtension)
            ? null
            : _registryOptions?.GetScopeByExtension(grammarExtension);
        _textMateInstallation?.SetGrammar(scope);
    }

    private void ApplyTextMateTheme()
    {
        if (_textMateInstallation is null || _registryOptions is null) return;
        _textMateInstallation.SetTheme(_registryOptions.LoadTheme(CurrentThemeName()));
    }

    private ThemeName CurrentThemeName() =>
        ActualThemeVariant == ThemeVariant.Dark ? ThemeName.DarkPlus : ThemeName.LightPlus;

    private void OnTextMateThemeApplied(object? sender, TextMateInstallation installation) =>
        ApplyEditorThemeColors(installation);

    private void ApplyEditorThemeColors(TextMateInstallation installation)
    {
        ApplyThemeBrush(installation, "editor.foreground", brush => SourceEditor.Foreground = brush);
        ApplyThemeBrush(installation, "editor.selectionBackground", brush => SourceEditor.TextArea.SelectionBrush = brush);
        ApplyThemeBrush(installation, "editorLineNumber.foreground", brush => SourceEditor.LineNumbersForeground = brush);
        SourceEditor.TextArea.Caret.CaretBrush = Brushes.Transparent;
    }

    private static void ApplyThemeBrush(
        TextMateInstallation installation,
        string colorKey,
        Action<IBrush> apply)
    {
        if (installation.TryGetThemeColor(colorKey, out var colorValue)
            && Color.TryParse(colorValue, out var color))
        {
            apply(new SolidColorBrush(color));
        }
    }

    private void DisposeTextMate()
    {
        if (_viewModel is not null) _viewModel.PropertyChanged -= OnViewModelPropertyChanged;
        _viewModel = null;
        if (_textMateInstallation is not null)
        {
            _textMateInstallation.AppliedTheme -= OnTextMateThemeApplied;
            _textMateInstallation.Dispose();
        }
        _textMateInstallation = null;
        _registryOptions = null;
    }
}
