package ai.suncode.service;

import ai.suncode.common.exception.BusinessException;
import ai.suncode.message.remote.*;
import org.springframework.stereotype.Service;

import java.time.Duration;
import java.time.Instant;
import java.util.UUID;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.TimeUnit;

import static ai.suncode.common.exception.ErrorCode.PARAM_INVALID;
import static ai.suncode.common.exception.ErrorCode.PAIRING_EXPIRED;
import static ai.suncode.common.exception.ErrorCode.UNAUTHORIZED;

@Service
public class RemoteAuthService {
    private static final Duration ACCESS_TOKEN_LIFETIME = Duration.ofHours(1);
    private final ConcurrentHashMap<String, String> mobileTokens = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<String, String> refreshTokens = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<String, PendingPairing> pairings = new ConcurrentHashMap<>();

    public String createPairing(String hostId, String displayName) {
        String payload = UUID.randomUUID().toString();
        pairings.put(payload, new PendingPairing(hostId, displayName, System.nanoTime() + TimeUnit.MINUTES.toNanos(5)));
        return payload;
    }

    public PairingExchangeData exchange(PairingExchangeRequest request) {
        if (request == null || request.pairingPayload() == null || request.pairingPayload().isBlank()) {
            throw new BusinessException(PARAM_INVALID, "pairingPayload is required");
        }
        PendingPairing pairing = pairings.remove(request.pairingPayload());
        if (pairing == null || pairing.expiresAtNanos() < System.nanoTime()) {
            throw new BusinessException(PAIRING_EXPIRED);
        }
        String access = UUID.randomUUID().toString();
        String refresh = UUID.randomUUID().toString();
        mobileTokens.put(access, pairing.hostId());
        refreshTokens.put(refresh, pairing.hostId());
        return new PairingExchangeData(
                access,
                refresh,
                Instant.now().plus(ACCESS_TOKEN_LIFETIME),
                new HostDto(pairing.hostId(), pairing.displayName(), "", "offline", 0, 0, null, null, null));
    }

    public TokenData refresh(RefreshTokenRequest request) {
        String hostId = request == null ? null : refreshTokens.get(request.refreshToken());
        if (hostId == null) {
            throw new BusinessException(UNAUTHORIZED, "invalid refresh token");
        }
        String access = UUID.randomUUID().toString();
        String refresh = UUID.randomUUID().toString();
        mobileTokens.put(access, hostId);
        refreshTokens.put(refresh, hostId);
        return new TokenData(access, refresh, Instant.now().plus(ACCESS_TOKEN_LIFETIME));
    }

    public String requireMobile(String authorization) {
        String token = bearer(authorization);
        if (!mobileTokens.containsKey(token)) {
            throw new BusinessException(UNAUTHORIZED, "invalid access token");
        }
        return token;
    }

    public String hostForMobileToken(String token) {
        String hostId = mobileTokens.get(token);
        if (hostId == null) {
            throw new BusinessException(UNAUTHORIZED, "invalid access token");
        }
        return hostId;
    }

    public void requireHostAccessToken(String token, String hostId) {
        if (hostId == null || !hostId.equals(hostForMobileToken(token))) {
            throw new BusinessException(UNAUTHORIZED, "mobile credential is not paired with this host");
        }
    }

    public boolean isPairedWith(String authorization, String hostId) {
        try {
            return hostId.equals(mobileTokens.get(requireMobile(authorization)));
        } catch (BusinessException error) {
            return false;
        }
    }

    public String requireDesktop(String authorization) {
        String token = bearer(authorization);
        if (token.isBlank()) {
            throw new BusinessException(UNAUTHORIZED, "desktop token is required");
        }
        return token;
    }

    public void logoutToken(String token) {
        mobileTokens.remove(token);
    }

    private static String bearer(String authorization) {
        if (authorization == null || !authorization.startsWith("Bearer ")) {
            return "";
        }
        return authorization.substring("Bearer ".length()).trim();
    }

}
