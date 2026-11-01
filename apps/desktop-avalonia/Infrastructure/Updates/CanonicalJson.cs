using System.Globalization;
using System.Text;
using System.Text.Json;

namespace SunCode.Desktop.Infrastructure.Updates;

// RFC 8785: UTF-16 property ordering, I-JSON strings, and ECMAScript number layout.
// The complete signed object is retained; DTO deserialization must not precede signing.
internal static class CanonicalJson
{
    private static readonly UTF8Encoding Utf8 = new(false, true);
    public static byte[] Encode(JsonElement element)
    {
        var result = new StringBuilder();
        Append(result, element);
        return Utf8.GetBytes(result.ToString());
    }
    private static void Append(StringBuilder result, JsonElement value)
    {
        switch (value.ValueKind)
        {
            case JsonValueKind.Object:
                result.Append('{');
                var properties = value.EnumerateObject().ToArray();
                if (properties.Select(p => p.Name).Distinct(StringComparer.Ordinal).Count() != properties.Length)
                    throw new UpdateException("invalid_catalog", "Update metadata contains duplicate properties.");
                var first = true;
                foreach (var property in properties.OrderBy(p => p.Name, StringComparer.Ordinal))
                {
                    if (!first) result.Append(','); first = false;
                    AppendString(result, property.Name); result.Append(':'); Append(result, property.Value);
                }
                result.Append('}'); break;
            case JsonValueKind.Array:
                result.Append('['); var separator = false;
                foreach (var item in value.EnumerateArray()) { if (separator) result.Append(','); separator = true; Append(result, item); }
                result.Append(']'); break;
            case JsonValueKind.String: AppendString(result, value.GetString()!); break;
            case JsonValueKind.Number:
                if (!value.TryGetDouble(out var number) || !double.IsFinite(number))
                    throw new UpdateException("invalid_catalog", "Update metadata contains an invalid number.");
                result.Append(Number(number)); break;
            case JsonValueKind.True: result.Append("true"); break;
            case JsonValueKind.False: result.Append("false"); break;
            case JsonValueKind.Null: result.Append("null"); break;
            default: throw new UpdateException("invalid_catalog", "Update metadata contains an invalid JSON value.");
        }
    }
    private static void AppendString(StringBuilder result, string text)
    {
        result.Append('"');
        for (var index = 0; index < text.Length; index++)
        {
            var c = text[index];
            switch (c)
            {
                case '"': result.Append("\\\""); break;
                case '\\': result.Append("\\\\"); break;
                case '\b': result.Append("\\b"); break;
                case '\t': result.Append("\\t"); break;
                case '\n': result.Append("\\n"); break;
                case '\f': result.Append("\\f"); break;
                case '\r': result.Append("\\r"); break;
                default:
                    if (c < 32) result.Append("\\u").Append(((int)c).ToString("x4", CultureInfo.InvariantCulture));
                    else if (char.IsHighSurrogate(c) && index + 1 < text.Length && char.IsLowSurrogate(text[index + 1]))
                        result.Append(c).Append(text[++index]);
                    else if (char.IsSurrogate(c)) throw new UpdateException("invalid_catalog", "Update metadata contains an invalid Unicode string.");
                    else result.Append(c);
                    break;
            }
        }
        result.Append('"');
    }
    internal static string Number(double value)
    {
        if (value == 0) return "0";
        var text = Math.Abs(value).ToString("R", CultureInfo.InvariantCulture);
        var split = text.Split('E', 'e');
        var mantissa = split[0];
        var exponent = split.Length == 2 ? int.Parse(split[1], CultureInfo.InvariantCulture) : 0;
        var point = mantissa.IndexOf('.');
        var digits = mantissa.Replace(".", "", StringComparison.Ordinal);
        var decimalPosition = (point < 0 ? mantissa.Length : point) + exponent;
        while (digits.Length > 1 && digits[0] == '0') { digits = digits[1..]; decimalPosition--; }
        var sign = value < 0 ? "-" : "";
        if (decimalPosition > 0 && decimalPosition <= 21)
            return sign + (digits.Length <= decimalPosition ? digits.PadRight(decimalPosition, '0') : digits.Insert(decimalPosition, "."));
        if (decimalPosition <= 0 && decimalPosition > -6)
            return sign + "0." + new string('0', -decimalPosition) + digits;
        var scientific = decimalPosition - 1;
        return sign + digits[0] + (digits.Length > 1 ? "." + digits[1..] : "") + "e"
            + (scientific >= 0 ? "+" : "") + scientific.ToString(CultureInfo.InvariantCulture);
    }
}
