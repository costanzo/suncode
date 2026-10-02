package ai.suncode.message.remote;

public record PairingExchangeRequest(
        String pairingCode,
        String deviceName) {
}
