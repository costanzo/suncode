package ai.suncode.message;

import com.fasterxml.jackson.annotation.JsonInclude;
import lombok.Getter;
import lombok.ToString;

@Getter
@ToString
@JsonInclude(JsonInclude.Include.NON_NULL)
public class ApiBaseRet<T>{
    private int code = 0;
    private String message;
    private T data;

    public ApiBaseRet() {
    }

    public ApiBaseRet(int code, String message) {
        this.code = code;
        this.message = message;
    }

    public ApiBaseRet(T data) {
        this.data = data;
    }

    public static ApiBaseRet<?> success() {
        return new ApiBaseRet<>();
    }

    public static <T> ApiBaseRet<T> success(T data) {
        return new ApiBaseRet<>(data);
    }

    public static ApiBaseRet<?> error(int code, String message) {
        return new ApiBaseRet<>(code, message);
    }
}
