import UIKit
import SwiftUI
import Shared

struct ComposeView: UIViewControllerRepresentable {
    let onScanPairing: (((String) -> Void) -> Void)

    func makeUIViewController(context: Self.Context) -> UIViewController {
        MainViewControllerKt.MainViewController(onScanPairing: onScanPairing)
    }

    func updateUIViewController(_ uiViewController: UIViewController, context: Self.Context) {}
}

struct ContentView: View {
    var body: some View {
        ComposeView { onResult in
            presentScanner(onResult: onResult)
        }
            .ignoresSafeArea()
    }

    private func presentScanner(onResult: @escaping (String) -> Void) {
        DispatchQueue.main.async {
            guard let presenter = topViewController() else { return }
            let scanner = QRCodeScannerViewController(onPayload: onResult)
            scanner.modalPresentationStyle = .fullScreen
            presenter.present(scanner, animated: true)
        }
    }
}

private func topViewController(
    from root: UIViewController? = UIApplication.shared.connectedScenes
        .compactMap { ($0 as? UIWindowScene)?.keyWindow }
        .first?.rootViewController
) -> UIViewController? {
    if let presented = root?.presentedViewController {
        return topViewController(from: presented)
    }
    if let navigation = root as? UINavigationController {
        return topViewController(from: navigation.visibleViewController)
    }
    if let tab = root as? UITabBarController {
        return topViewController(from: tab.selectedViewController)
    }
    return root
}
