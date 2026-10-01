package ai.suncode.message.remote;

import com.fasterxml.jackson.annotation.JsonAlias;

public record DesktopPairingRequest(
        @JsonAlias("pairing_code") String pairingCode,
        String displayName,
        @JsonAlias("desktop_version") String desktopVersion) {
}
