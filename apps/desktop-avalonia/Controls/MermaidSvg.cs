using System.Globalization;
using System.Text.RegularExpressions;
using System.Xml.Linq;
using Avalonia.Media;

namespace SunCode.Desktop.Controls;

/// <summary>Resolves the CSS variables and sRGB mixes emitted by Mermaider for the SVG desktop renderer.</summary>
internal static partial class MermaidSvg
{
    [GeneratedRegex(@"(--[\w-]+)\s*:\s*([^;{}]+)", RegexOptions.CultureInvariant)]
    private static partial Regex Variables();

    internal static string Materialize(string svg)
    {
        var document = XDocument.Parse(svg);
        var root = document.Root ?? throw new FormatException("Missing SVG root.");
        var variables = new Dictionary<string, string>(StringComparer.Ordinal);
        foreach (var style in root.Descendants().Where(element => element.Name.LocalName == "style"))
            ReadVariables(style.Value, variables);
        ReadVariables(root.Attribute("style")?.Value ?? string.Empty, variables);
        // Keep the desktop's established content sizes instead of browser rem/calc semantics.
        variables["--fs-m"] = "14px";
        variables["--fs-s"] = "12px";
        variables["--fs-xs"] = "11px";
        variables["--fs-l"] = "15px";

        foreach (var element in root.DescendantsAndSelf())
        {
            foreach (var attribute in element.Attributes().ToArray())
                if (attribute.Name.LocalName is "fill" or "stroke" or "stop-color" or "flood-color" or "color" or "font-size" or "font-family" or "style")
                    attribute.Value = Resolve(attribute.Value, variables, 0);
            if (element.Name.LocalName == "style")
                element.Value = Resolve(element.Value, variables, 0);
        }
        return document.ToString(SaveOptions.DisableFormatting);
    }

    private static void ReadVariables(string text, Dictionary<string, string> variables)
    {
        foreach (Match match in Variables().Matches(text))
            variables[match.Groups[1].Value] = match.Groups[2].Value.Trim();
    }

    private static string Resolve(string value, Dictionary<string, string> variables, int depth)
    {
        if (depth > 32) throw new FormatException("Circular SVG theme variable.");
        value = ReplaceFunction(value, "var(", inner =>
        {
            var comma = TopLevelComma(inner);
            var key = (comma < 0 ? inner : inner[..comma]).Trim();
            var replacement = variables.TryGetValue(key, out var found) ? found
                : comma >= 0 ? inner[(comma + 1)..].Trim()
                : throw new FormatException($"Unknown SVG theme variable: {key}");
            return Resolve(replacement, variables, depth + 1);
        });
        return ReplaceFunction(value, "color-mix(", inner => Mix(Resolve(inner, variables, depth + 1)));
    }

    private static string ReplaceFunction(string value, string function, Func<string, string> resolve)
    {
        var start = value.IndexOf(function, StringComparison.Ordinal);
        while (start >= 0)
        {
            var open = start + function.Length - 1;
            var nesting = 1;
            var end = open + 1;
            for (; end < value.Length && nesting > 0; end++)
            {
                if (value[end] == '(') nesting++;
                else if (value[end] == ')') nesting--;
            }
            if (nesting != 0) throw new FormatException("Unclosed SVG theme expression.");
            var replacement = resolve(value[(open + 1)..(end - 1)]);
            value = value[..start] + replacement + value[end..];
            start = value.IndexOf(function, start + replacement.Length, StringComparison.Ordinal);
        }
        return value;
    }

    private static int TopLevelComma(string value)
    {
        var nesting = 0;
        for (var index = 0; index < value.Length; index++)
        {
            if (value[index] == '(') nesting++;
            else if (value[index] == ')') nesting--;
            else if (value[index] == ',' && nesting == 0) return index;
        }
        return -1;
    }

    private static string Mix(string expression)
    {
        var firstComma = TopLevelComma(expression);
        if (firstComma < 0 || expression[..firstComma].Trim() != "in srgb")
            throw new FormatException("Unsupported SVG color space.");
        var colors = expression[(firstComma + 1)..].Trim();
        var separator = TopLevelComma(colors);
        if (separator < 0) throw new FormatException("Missing SVG mix color.");
        var first = colors[..separator].Trim();
        var second = colors[(separator + 1)..].Trim();
        var weight = 0.5;
        if (first.EndsWith('%'))
        {
            var space = first.LastIndexOf(' ');
            weight = double.Parse(first[(space + 1)..^1], CultureInfo.InvariantCulture) / 100;
            first = first[..space].Trim();
        }
        if (!Color.TryParse(first, out var left) || !Color.TryParse(second, out var right))
            throw new FormatException("Unsupported SVG mix color.");
        var alpha = left.A * weight + right.A * (1 - weight);
        byte Channel(byte a, byte b) => alpha == 0 ? (byte)0
            : (byte)Math.Clamp(Math.Round((a * left.A * weight + b * right.A * (1 - weight)) / alpha), 0, 255);
        var r = Channel(left.R, right.R);
        var g = Channel(left.G, right.G);
        var b = Channel(left.B, right.B);
        return alpha >= 254.5 ? $"#{r:X2}{g:X2}{b:X2}"
            : FormattableString.Invariant($"rgba({r},{g},{b},{alpha / 255:0.###})");
    }
}
