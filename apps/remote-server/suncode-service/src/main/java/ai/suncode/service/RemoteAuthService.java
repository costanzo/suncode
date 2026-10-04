package ai.suncode.service;

import ai.suncode.common.exception.BusinessException;
import ai.suncode.message.remote.*;
import org.springframework.stereotype.Service;
import org.springframework.beans.factory.annotation.Value;

import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.SecureRandom;
import java.time.Duration;
import java.time.Instant;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.TimeUnit;
import java.util.UUID;

import static ai.suncode.common.exception.ErrorCode.PARAM_INVALID;
import static ai.suncode.common.exception.ErrorCode.PAIRING_EXPIRED;
import static ai.suncode.common.exception.ErrorCode.UNAUTHORIZED;

@Service
public class RemoteAuthService {
    private static final Duration ACCESS_TOKEN_LIFETIME = Duration.ofHours(1);
    private static final int HOST_ID_LENGTH = 10;
    private static final int MOBILE_PAIRING_CODE_LENGTH = 6;
    private static final String HOST_ID_ALPHABET = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz";
    private static final String DIGITS = "0123456789";
    private static final SecureRandom RANDOM = new SecureRandom();
    private final ConcurrentHashMap<String, String> mobileTokens = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<String, String> refreshTokens = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<String, String> desktopTokens = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<String, String> desktopRefreshTokens = new ConcurrentHashMap<>();
    private final ConcurrentHashMap<String, PendingPairing> pairings = new ConcurrentHashMap<>();
    private final String desktopPairingCode;

    public RemoteAuthService(
            @Value("${suncode.remote.desktop-pairing-code:}") String desktopPairingCode) {
        this.desktopPairingCode = desktopPairingCode == null ? "" : desktopPairingCode;
    }

    public DesktopPairingPayload createDesktopPairing(DesktopPairingRequest request) {
        if (request == null || !matchesPairingCode(request.pairingCode())) {
            throw new BusinessException(UNAUTHORIZED, "invalid desktop pairing code");
        }
        String hostId;
        do {
            hostId = randomToken(HOST_ID_ALPHABET, HOST_ID_LENGTH);
        } while (desktopTokens.containsKey(hostId));
        String desktopToken = UUID.randomUUID().toString();
        String desktopRefresh = UUID.randomUUID().toString();
        desktopTokens.put(hostId, desktopToken);
        desktopRefreshTokens.put(desktopRefresh, hostId);
        String displayName = request.displayName() == null || request.displayName().isBlank()
                ? hostId : request.displayName();
        String mobilePairingPayload = createPairing(hostId, displayName);
        return new DesktopPairingPayload(
                hostId,
                desktopToken,
                desktopRefresh,
                Instant.now().plus(ACCESS_TOKEN_LIFETIME),
                mobilePairingPayload);
    }

    public String createPairing(String hostId, String displayName) {
        String payload;
        do {
            payload = randomToken(DIGITS, MOBILE_PAIRING_CODE_LENGTH);
        } while (pairings.containsKey(payload));
        pairings.put(payload, new PendingPairing(hostId, displayName, System.nanoTime() + TimeUnit.MINUTES.toNanos(5)));
        return payload;
    }

    public PairingExchangeData exchange(PairingExchangeRequest request) {
        if (request == null || request.pairingCode() == null || request.pairingCode().isBlank()) {
            throw new BusinessException(PARAM_INVALID, "pairingCode is required");
        }
        PendingPairing pairing = pairings.remove(request.pairingCode());
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
                new HostDto(pairing.hostId(), pairing.displayName(), "offline", null, null));
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

    public TokenData refreshMobile(String hostId, RefreshTokenRequest request) {
        String token = request == null ? null : request.refreshToken();
        String tokenHost = token == null ? null : refreshTokens.get(token);
        if (tokenHost == null || !tokenHost.equals(hostId)) {
            throw new BusinessException(UNAUTHORIZED, "invalid refresh token");
        }
        refreshTokens.remove(token, hostId);
        return refresh(request);
    }

    public TokenData refreshDesktop(String hostId, RefreshTokenRequest request) {
        String tokenHost = request == null ? null : desktopRefreshTokens.remove(request.refreshToken());
        if (tokenHost == null || !tokenHost.equals(hostId)) {
            throw new BusinessException(UNAUTHORIZED, "invalid desktop refresh token");
        }
        String access = UUID.randomUUID().toString();
        String refresh = UUID.randomUUID().toString();
        desktopTokens.put(hostId, access);
        desktopRefreshTokens.put(refresh, hostId);
        return new TokenData(access, refresh, Instant.now().plus(ACCESS_TOKEN_LIFETIME));
    }

    public void requirePairingHost(String hostId, String pairingCode) {
        PendingPairing pairing = pairings.get(pairingCode);
        if (pairing == null || !pairing.hostId().equals(hostId)) {
            throw new BusinessException(UNAUTHORIZED, "pairing code does not belong to host");
        }
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

    public String requireDesktop(String authorization, String hostId) {
        String token = bearer(authorization);
        if (token.isBlank() || hostId == null || !token.equals(desktopTokens.get(hostId))) {
            throw new BusinessException(UNAUTHORIZED, "desktop token is required");
        }
        return token;
    }

    private boolean matchesPairingCode(String candidate) {
        if (desktopPairingCode.isBlank() || candidate == null) {
            return false;
        }
        return MessageDigest.isEqual(
                desktopPairingCode.getBytes(StandardCharsets.UTF_8),
                candidate.getBytes(StandardCharsets.UTF_8));
    }

    private static String randomToken(String alphabet, int length) {
        StringBuilder token = new StringBuilder(length);
        for (int index = 0; index < length; index++) {
            token.append(alphabet.charAt(RANDOM.nextInt(alphabet.length())));
        }
        return token.toString();
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
