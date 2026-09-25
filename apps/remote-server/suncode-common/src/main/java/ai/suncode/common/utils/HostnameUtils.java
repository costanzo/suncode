package ai.suncode.common.utils;

import lombok.Getter;
import lombok.extern.slf4j.Slf4j;

import java.net.InetAddress;

@Slf4j
public class HostnameUtils {
    @Getter
    private static String hostname;
    @Getter
    private static String obfuscatedHostname;

    private static final String UNKNOWN_HOST = "unknown_host";

    static {
        try {
            hostname = hostname();
            obfuscatedHostname = calculateObfuscatedHostname(hostname);
        } catch (Exception e) {
            log.error("Error while initializing HostnameUtil", e);
            hostname = UNKNOWN_HOST;
            obfuscatedHostname = UNKNOWN_HOST;
        }
    }

    private static String calculateObfuscatedHostname(String input) {
        try {
            if (input != null && !input.isEmpty() && !input.trim().isEmpty()) {
                StringBuilder sb = new StringBuilder();
                char[] temp = input.trim().toCharArray();
                int length = temp.length;
                for (int i = 0; i < length - 1; i++) {
                    if (Character.isDigit(temp[i]) || Character.isDigit(temp[i + 1])) {
                        sb.append(temp[i]);
                    }
                }
                sb.append(temp[length - 1]);
                return sb.toString();
            }
        } catch (Exception e) {
            log.error("Error while calculating obfuscated hostname", e);
        }
        return null;
    }

    private static String hostname() {
        try {
            InetAddress inetAddress = InetAddress.getLocalHost();
            return inetAddress.getHostName();
        } catch (Exception e) {
            log.error("Error while getting hostname", e);
        }
        return null;
    }
}
