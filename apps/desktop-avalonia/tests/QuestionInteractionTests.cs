using System.Text.Json;
using SunCode.Desktop.Models;
using SunCode.Desktop.ViewModels;
using SunCode.Sdk.Models;

namespace SunCode.Desktop.Tests;

public sealed class QuestionInteractionTests
{
    [Fact]
    public async Task DisconnectedSdkReportsAnInlineErrorAndKeepsTheDraft()
    {
        using var model = new DesktopViewModel();
        var request = CreateRequest();
        model.PendingQuestion = request;
        request.Questions[0].CustomAnswer = "Retain this answer";
        await model.ReplyQuestionAsync();
        Assert.Same(request, model.PendingQuestion);
        Assert.True(request.HasError);
        Assert.True(request.CanSubmit);
        Assert.Equal(["Retain this answer"], request.Questions[0].Answers);
    }

    [Fact]
    public void SingleChoiceAndCustomAnswersRemainMutuallyExclusive()
    {
        var question = CreateRequest().Questions[0];
        question.Options[0].IsSelected = true;
        question.Options[1].IsSelected = true;
        Assert.Equal(["Second"], question.Answers);
        question.CustomAnswer = "  Own answer  ";
        Assert.DoesNotContain(question.Options, option => option.IsSelected);
        Assert.Equal(["Own answer"], question.Answers);
        question.Options[0].IsSelected = true;
        Assert.Empty(question.CustomAnswer);
        Assert.Equal(["First"], question.Answers);
    }

    [Fact]
    public void MultipleChoiceRetainsSelectionsAlongsideCustomText()
    {
        var request = CreateRequest(multiple: true);
        var question = request.Questions[0];
        question.Options[0].IsSelected = true;
        question.Options[1].IsSelected = true;
        question.CustomAnswer = "Additional context";
        Assert.Equal(["First", "Second", "Additional context"], question.Answers);
        Assert.Equal(1, request.AnsweredCount);
        Assert.True(request.CanSubmit);
        Assert.All(question.Options, option => Assert.True(option.Multiple));
    }

    [Fact]
    public void RefreshOfTheSameRequestRetainsDraftAndResolutionState()
    {
        using var model = new DesktopViewModel();
        var request = CreateRequest();
        model.PendingQuestion = request;
        request.Questions[0].CustomAnswer = "Keep this draft";
        Assert.True(request.TryBeginResolution(skip: false));
        model.PendingQuestion = CreateRequest();
        Assert.Same(request, model.PendingQuestion);
        Assert.Equal("Keep this draft", model.PendingQuestion.Questions[0].CustomAnswer);
        Assert.True(model.PendingQuestion.IsSubmitting);
    }

    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task ResolutionDisablesEditingAndRejectsConcurrentActions(bool skip)
    {
        using var model = new DesktopViewModel();
        var question = CreateRequest();
        model.PendingQuestion = question;
        question.Questions[0].CustomAnswer = "Answer";
        var pending = new TaskCompletionSource();
        var attempts = 0;
        Task Resolve() { attempts++; return pending.Task; }
        var first = model.ResolveQuestionAsync(question, skip, Resolve);
        Assert.False(question.IsEditable);
        Assert.False(question.CanSubmit);
        Assert.Equal(skip, question.IsSkipping);
        Assert.Equal(!skip, question.IsSubmitting);
        await model.ResolveQuestionAsync(question, !skip, Resolve);
        Assert.Equal(1, attempts);
        pending.SetResult();
        await first;
        Assert.Null(model.PendingQuestion);
        Assert.False(question.IsResolving);
    }

    [Theory]
    [InlineData(false)]
    [InlineData(true)]
    public async Task FailedResolutionRetainsAnswersAndAllowsRetry(bool skip)
    {
        using var model = new DesktopViewModel();
        var question = CreateRequest();
        model.PendingQuestion = question;
        question.Questions[0].CustomAnswer = "Retained answer";
        await model.ResolveQuestionAsync(question, skip, () => Task.FromException(new InvalidOperationException("Unavailable")));
        Assert.Same(question, model.PendingQuestion);
        Assert.True(question.HasError);
        Assert.True(question.IsEditable);
        Assert.True(question.CanSubmit);
        Assert.Equal(["Retained answer"], question.Questions[0].Answers);
        await model.ResolveQuestionAsync(question, skip, () => Task.CompletedTask);
        Assert.Null(model.PendingQuestion);
        Assert.False(question.HasError);
    }

    [Fact]
    public async Task PreviousResolutionCannotClearTheNextRequest()
    {
        using var model = new DesktopViewModel();
        var previous = CreateRequest();
        model.PendingQuestion = previous;
        var pending = new TaskCompletionSource();
        var resolution = model.ResolveQuestionAsync(previous, skip: true, () => pending.Task);
        var next = CreateRequest("next");
        model.PendingQuestion = next;
        pending.SetResult();
        await resolution;
        Assert.Same(next, model.PendingQuestion);
    }

    [Theory]
    [InlineData(AgentEventTypes.QuestionReplied)]
    [InlineData(AgentEventTypes.QuestionRejected)]
    public void PreviousRequestEventsCannotClearTheCurrentQuestion(string type)
    {
        using var model = new DesktopViewModel();
        var current = CreateRequest("next");
        model.PendingQuestion = current;
        var payload = JsonSerializer.Deserialize<AgentEventPayload>("""{"request_id":"previous","turn_id":"turn-1"}""")!;
        model.ApplyEvent(new AgentEvent("event", "2026-11-08T00:00:00Z", type, payload), live: true);
        Assert.Same(current, model.PendingQuestion);
        var currentPayload = JsonSerializer.Deserialize<AgentEventPayload>("""{"request_id":"next","turn_id":"turn"}""")!;
        model.ApplyEvent(new AgentEvent("resolved", "2026-11-08T00:00:01Z", type, currentPayload), live: true);
        Assert.Null(model.PendingQuestion);
    }

    internal static PendingQuestionItem CreateRequest(string id = "question", bool multiple = false, int count = 1)
    {
        var request = new PendingQuestionItem(id, "turn", "tool");
        for (var index = 0; index < count; index++)
        {
            var question = new QuestionPromptItem($"Topic {index + 1}",
                "Which behavior should be used when the available space becomes too narrow to show the complete request and its details while retaining the current selection and reading position?",
                multiple || index % 2 == 1, allowCustom: index != 2);
            question.Options.Add(new QuestionOptionItem("First", "Preserve the current selection, filters, and complete content while adapting the layout to the available space."));
            question.Options.Add(new QuestionOptionItem("Second", "Keep the detail readable and restore the list's reading position after returning from the selected request."));
            request.Questions.Add(question);
        }
        return request;
    }
}
