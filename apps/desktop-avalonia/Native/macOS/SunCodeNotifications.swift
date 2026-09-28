import Foundation
import UserNotifications
import os.log

public typealias ActivationCallback = @convention(c) (UnsafePointer<CChar>?) -> Void
public typealias DeliveryCallback = @convention(c) (Int32) -> Void

private let logger = OSLog(subsystem: "dev.suncode.desktop", category: "notifications")

private final class SunCodeNotificationDelegate: NSObject, UNUserNotificationCenterDelegate {
    var callback: ActivationCallback?

    func userNotificationCenter(
    _ center: UNUserNotificationCenter,
        willPresent notification: UNNotification,
        withCompletionHandler completionHandler: @escaping (UNNotificationPresentationOptions) -> Void
    ) {
        completionHandler([.banner, .sound, .list])
    }

    func userNotificationCenter(
        _ center: UNUserNotificationCenter,
        didReceive response: UNNotificationResponse,
        withCompletionHandler completionHandler: @escaping () -> Void
    ) {
        if let activation = response.notification.request.content.userInfo["activation"] as? String {
            activation.withCString { pointer in callback?(pointer) }
        }
        completionHandler()
    }
}

private let notificationDelegate = SunCodeNotificationDelegate()

@_cdecl("suncode_notifications_initialize")
public func initializeNotifications(_ callback: ActivationCallback?) {
    notificationDelegate.callback = callback
    let center = UNUserNotificationCenter.current()
    center.delegate = notificationDelegate
    center.requestAuthorization(options: [.alert, .sound]) { granted, error in
        if let error = error {
            os_log("Failed to request authorization: %@", log: logger, type: .error, error.localizedDescription)
        } else {
            os_log("Authorization granted: %@", log: logger, type: .info, granted ? "Yes" : "No")
        }
    }
}

@_cdecl("suncode_notifications_show")
public func showNotification(
    _ identifier: UnsafePointer<CChar>?,
    _ title: UnsafePointer<CChar>?,
    _ body: UnsafePointer<CChar>?,
    _ activation: UnsafePointer<CChar>?,
    _ callback: DeliveryCallback?
) {
    guard let identifier, let title, let body, let activation else {
        callback?(0)
        return
    }
    let identifierValue = String(cString: identifier)
    let titleValue = String(cString: title)
    let bodyValue = String(cString: body)
    let activationValue = String(cString: activation)
    let center = UNUserNotificationCenter.current()

    func deliver() {
        let content = UNMutableNotificationContent()
        content.title = titleValue
        content.body = bodyValue
        content.userInfo = ["activation": activationValue]
        center.add(UNNotificationRequest(identifier: identifierValue, content: content, trigger: nil)) { error in
            if let error = error {
                os_log("Failed to deliver notification: %@", log: logger, type: .error, error.localizedDescription)
                callback?(0)
            } else {
                callback?(1)
            }
        }
    }

    center.getNotificationSettings { settings in
        switch settings.authorizationStatus {
        case .authorized, .provisional, .ephemeral:
            deliver()
        case .notDetermined:
            center.requestAuthorization(options: [.alert, .sound]) { granted, error in
                if let error = error {
                    os_log("Failed to request authorization: %@", log: logger, type: .error, error.localizedDescription)
                }
                granted ? deliver() : callback?(0)
            }
        case .denied:
            os_log("Notification authorization denied", log: logger, type: .error)
            callback?(0)
        @unknown default:
            os_log("Unknown notification authorization status", log: logger, type: .error)
            callback?(0)
        }
    }
}
