import Foundation
import UserNotifications

public typealias ActivationCallback = @convention(c) (UnsafePointer<CChar>?) -> Void
public typealias DeliveryCallback = @convention(c) (Int32) -> Void

private final class SunCodeNotificationDelegate: NSObject, UNUserNotificationCenterDelegate {
    var callback: ActivationCallback?

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
        center.add(UNNotificationRequest(identifier: identifierValue, content: content, trigger: nil)) {
            callback?($0 == nil ? 1 : 0)
        }
    }

    center.getNotificationSettings { settings in
        switch settings.authorizationStatus {
        case .authorized, .provisional, .ephemeral:
            deliver()
        case .notDetermined:
            center.requestAuthorization(options: [.alert, .sound]) { granted, _ in
                granted ? deliver() : callback?(0)
            }
        default:
            callback?(0)
        }
    }
}
