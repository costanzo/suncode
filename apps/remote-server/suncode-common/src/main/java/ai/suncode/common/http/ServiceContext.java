package ai.suncode.common.http;

import ai.suncode.common.exception.BusinessException;
import ai.suncode.message.remote.ClientType;

import static ai.suncode.common.exception.ErrorCode.INTERNAL_ERROR;

/** Immutable context for the current HTTP request. */
public record ServiceContext(
        ClientType clientType,
        String hostId,
        String token,
        String requestId) {
    private static final ThreadLocal<ServiceContext> CURRENT = new ThreadLocal<>();

    public static ServiceContext current() {
        ServiceContext context = CURRENT.get();
        if (context == null) {
            throw new BusinessException(INTERNAL_ERROR, "ServiceContext is not available for this request");
        }
        return context;
    }

    public static void setCurrent(ServiceContext context) {
        CURRENT.set(context);
    }

    public static void authenticate(ClientType clientType, String hostId, String token) {
        ServiceContext context = current();
        CURRENT.set(new ServiceContext(clientType, hostId, token, context.requestId()));
    }

    public static void clear() {
        CURRENT.remove();
    }
}
