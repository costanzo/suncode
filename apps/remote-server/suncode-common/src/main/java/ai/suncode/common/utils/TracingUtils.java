package ai.suncode.common.utils;

public class TracingUtils {
    public static String createNewLogId() {
        String logId = String.valueOf(System.currentTimeMillis());
        logId = logId.substring(0, 10) + "_" + logId.substring(10);
        return logId + "_" + Thread.currentThread().getId() + "_" + HostnameUtils.getObfuscatedHostname();
    }

    public static String createNewRequestId() {
        return String.valueOf(System.currentTimeMillis());
    }
}
