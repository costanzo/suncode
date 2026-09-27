package ai.suncode.message.remote;

public record PairingExchangeRequest(
        String pairingPayload,
        String deviceName,
        String devicePublicKey,
        String clientNonce) {
}
