# Markdown

`MarkdownSpecimen.jsx` is a static semantic-HTML specimen of the expected post-render Markdown surface. It intentionally has no Markdown parser or fixture dependency. The code-block examples cover JavaScript, Python, Bash, HTML, CSS, JSON, Java, Go, and Rust using `token-*` spans to model the classes a future syntax highlighter should emit. Each fenced code specimen includes an accessible copy action that copies the code element's plain text. `Markdown.css` owns every Markdown-specific visual rule; global review styles must not define `.markdown*` selectors.
