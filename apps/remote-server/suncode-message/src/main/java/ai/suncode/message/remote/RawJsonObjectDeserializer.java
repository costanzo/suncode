package ai.suncode.message.remote;

import com.fasterxml.jackson.core.JsonFactory;
import com.fasterxml.jackson.core.JsonGenerator;
import com.fasterxml.jackson.core.JsonParser;
import com.fasterxml.jackson.core.JsonToken;
import com.fasterxml.jackson.databind.DeserializationContext;
import com.fasterxml.jackson.databind.JsonDeserializer;
import com.fasterxml.jackson.databind.JsonMappingException;

import java.io.IOException;
import java.io.StringWriter;

/** Keeps an object-valued JSON property as raw JSON text without building a tree. */
public class RawJsonObjectDeserializer extends JsonDeserializer<String> {
    @Override
    public String deserialize(JsonParser parser, DeserializationContext context) throws IOException {
        if (parser.currentToken() != JsonToken.START_OBJECT) {
            throw JsonMappingException.from(parser, "payload must be a JSON object");
        }
        StringWriter writer = new StringWriter();
        try (JsonGenerator generator = new JsonFactory().createGenerator(writer)) {
            generator.copyCurrentStructure(parser);
        }
        return writer.toString();
    }
}
