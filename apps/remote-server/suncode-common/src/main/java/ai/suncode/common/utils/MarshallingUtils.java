package ai.suncode.common.utils;

import ai.suncode.common.exception.BusinessException;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.SerializationFeature;

import static ai.suncode.common.exception.ErrorCode.INTERNAL_ERROR;

public class MarshallingUtils {
    private static final ObjectMapper OBJECT_MAPPER;

    static {
        OBJECT_MAPPER = new ObjectMapper().findAndRegisterModules();
        OBJECT_MAPPER.configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false);
        OBJECT_MAPPER.configure(SerializationFeature.FAIL_ON_EMPTY_BEANS, false);
        OBJECT_MAPPER.configure(SerializationFeature.WRITE_DATES_AS_TIMESTAMPS, false);
        OBJECT_MAPPER.setDefaultPropertyInclusion(JsonInclude.Include.NON_NULL);
    }

    public static String toJson(Object o) {
        try {
            return OBJECT_MAPPER.writeValueAsString(o);
        } catch (Exception e) {
            throw new BusinessException(INTERNAL_ERROR, "JSON serialization failed", e);
        }
    }

    public static <T> T fromJson(String json, Class<T> clazz) {
        try {
            return OBJECT_MAPPER.readValue(json, clazz);
        } catch (Exception e) {
            throw new BusinessException(INTERNAL_ERROR, "JSON deserialization failed", e);
        }
    }

    public static <T> T fromJson(byte[] json, Class<T> clazz) {
        try {
            return OBJECT_MAPPER.readValue(json, clazz);
        } catch (Exception e) {
            throw new BusinessException(INTERNAL_ERROR, "JSON deserialization failed", e);
        }
    }

    public static <T> T convertValue(Object value, Class<T> clazz) {
        try {
            return OBJECT_MAPPER.convertValue(value, clazz);
        } catch (Exception e) {
            throw new BusinessException(INTERNAL_ERROR, "JSON conversion failed", e);
        }
    }
}
