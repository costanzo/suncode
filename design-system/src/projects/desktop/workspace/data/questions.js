// Illustrative question requests for the desktop Review surface.
export const singleQuestion = [
  {
    header: "Responsive layout",
    question: "Which responsive behavior should the focused trace use?",
    multiple: false,
    custom: true,
    options: [
      {
        label: "Stack list above detail",
        description: "Keep the focused trace easy to scan on narrow screens.",
      },
      {
        label: "Keep a narrow split view",
        description: "Preserve side-by-side context when the viewport allows it.",
      },
    ],
  },
];

export const longQuestions = [
  {
    ...singleQuestion[0],
    question:
      "When the project window becomes too narrow to show the trace list and the selected request side by side, how should the focused trace adapt while preserving the user's current selection, filters, and reading position?",
    options: [
      {
        label: "Stack the list above the selected request",
        description:
          "Keep both regions available. Give the selected request the remaining vertical space, and retain the list's selection and scroll position as the window changes size.",
      },
      {
        label: "Show the selected request with a back action",
        description:
          "Use the full available width for the request. Returning to the list restores its filters, selection, and previous reading position.",
      },
      {
        label: "Keep the split view while space allows",
        description:
          "Preserve side-by-side inspection until the detail reaches its minimum readable width, then switch to a stacked layout.",
      },
    ],
  },
  {
    header: "Verification",
    question:
      "Which states should the implementation cover before the responsive trace is ready for review?",
    multiple: true,
    custom: true,
    options: [
      {
        label: "Loading and empty results",
        description:
          "Retain the selected request identity while loading and provide a clear empty state when filters match no requests.",
      },
      {
        label: "Long request and response content",
        description:
          "Verify long paths, multiline content, large payloads, and independent scrolling without hiding the request controls.",
      },
      {
        label: "Failure and recovery",
        description:
          "Retain the current selection after a request fails and make retry available in the detail region.",
      },
      {
        label: "Keyboard navigation",
        description:
          "Check list navigation, focus transfer into detail, and focus restoration after returning to the list.",
      },
    ],
  },
  {
    header: "Delivery scope",
    question: "Where should this layout change be implemented first?",
    multiple: false,
    custom: false,
    options: [
      {
        label: "Design-system specimen",
        description: "Review the interaction and layout before implementing the Avalonia client.",
      },
      {
        label: "Design system and Avalonia",
        description: "Implement the approved layout across both surfaces in the same delivery.",
      },
    ],
  },
];

export const eightQuestions = [
  ...longQuestions,
  {
    ...singleQuestion[0],
    header: "Initial selection",
    question: "Which request should be selected when the trace opens?",
  },
  {
    ...longQuestions[1],
    header: "Filters",
    question: "Which filter states should be preserved when the window is resized?",
  },
  {
    ...longQuestions[2],
    header: "Theme review",
    question: "Which review pass should cover the light and dark themes?",
  },
  {
    ...singleQuestion[0],
    header: "Return behavior",
    question: "How should the user return to the request list after inspecting a detail?",
  },
  {
    ...longQuestions[1],
    header: "Final checks",
    question: "Which checks should be included in the final review?",
  },
];
