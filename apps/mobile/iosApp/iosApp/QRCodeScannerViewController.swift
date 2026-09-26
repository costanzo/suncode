import AVFoundation
import UIKit

/// One-shot QR scanner used by the remote pairing flow.
final class QRCodeScannerViewController: UIViewController, AVCaptureMetadataOutputObjectsDelegate {
    private let onPayload: (String) -> Void
    private var captureSession: AVCaptureSession?
    private var previewLayer: AVCaptureVideoPreviewLayer?
    private var didFinish = false

    init(onPayload: @escaping (String) -> Void) {
        self.onPayload = onPayload
        super.init(nibName: nil, bundle: nil)
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        view.backgroundColor = .black

        let titleLabel = UILabel()
        titleLabel.text = "Pair a new Desktop"
        titleLabel.textColor = .white
        titleLabel.font = .preferredFont(forTextStyle: .headline)

        let subtitleLabel = UILabel()
        subtitleLabel.text = "Use the camera to scan the one-time code shown by SunCode Desktop."
        subtitleLabel.textColor = UIColor.white.withAlphaComponent(0.78)
        subtitleLabel.font = .preferredFont(forTextStyle: .subheadline)
        subtitleLabel.numberOfLines = 0

        let copyStack = UIStackView(arrangedSubviews: [titleLabel, subtitleLabel])
        copyStack.axis = .vertical
        copyStack.spacing = 6
        copyStack.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(copyStack)

        let scannerFrame = QRScannerFrameView()
        scannerFrame.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(scannerFrame)

        let hintLabel = UILabel()
        hintLabel.text = "Align QR code inside the frame"
        hintLabel.textColor = UIColor.white.withAlphaComponent(0.78)
        hintLabel.font = .preferredFont(forTextStyle: .caption1)
        hintLabel.textAlignment = .center
        hintLabel.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(hintLabel)

        let closeButton = UIButton(type: .system)
        closeButton.setTitle("Cancel", for: .normal)
        closeButton.setTitleColor(.white, for: .normal)
        closeButton.backgroundColor = UIColor.black.withAlphaComponent(0.55)
        closeButton.layer.cornerRadius = 10
        closeButton.contentEdgeInsets = UIEdgeInsets(top: 9, left: 14, bottom: 9, right: 14)
        closeButton.addTarget(self, action: #selector(cancel), for: .touchUpInside)
        closeButton.translatesAutoresizingMaskIntoConstraints = false
        view.addSubview(closeButton)
        NSLayoutConstraint.activate([
            copyStack.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor, constant: 24),
            copyStack.leadingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.leadingAnchor, constant: 20),
            copyStack.trailingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.trailingAnchor, constant: -20),
            scannerFrame.centerXAnchor.constraint(equalTo: view.centerXAnchor),
            scannerFrame.centerYAnchor.constraint(equalTo: view.centerYAnchor),
            scannerFrame.leadingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.leadingAnchor, constant: 38),
            scannerFrame.trailingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.trailingAnchor, constant: -38),
            scannerFrame.heightAnchor.constraint(equalTo: scannerFrame.widthAnchor),
            hintLabel.topAnchor.constraint(equalTo: scannerFrame.bottomAnchor, constant: 14),
            hintLabel.leadingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.leadingAnchor, constant: 20),
            hintLabel.trailingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.trailingAnchor, constant: -20),
            closeButton.topAnchor.constraint(equalTo: view.safeAreaLayoutGuide.topAnchor, constant: 16),
            closeButton.trailingAnchor.constraint(equalTo: view.safeAreaLayoutGuide.trailingAnchor, constant: -16),
        ])

        switch AVCaptureDevice.authorizationStatus(for: .video) {
        case .authorized:
            configureCaptureSession()
        case .notDetermined:
            AVCaptureDevice.requestAccess(for: .video) { [weak self] granted in
                DispatchQueue.main.async {
                    guard let self else { return }
                    if granted {
                        self.configureCaptureSession()
                    } else {
                        self.showPermissionMessage()
                    }
                }
            }
        case .denied, .restricted:
            showPermissionMessage()
        @unknown default:
            showPermissionMessage()
        }
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        if let captureSession, !captureSession.isRunning {
            DispatchQueue.global(qos: .userInitiated).async {
                captureSession.startRunning()
            }
        }
    }

    override func viewDidLayoutSubviews() {
        super.viewDidLayoutSubviews()
        previewLayer?.frame = view.bounds
    }

    override func viewWillDisappear(_ animated: Bool) {
        super.viewWillDisappear(animated)
        guard let captureSession, captureSession.isRunning else { return }
        DispatchQueue.global(qos: .userInitiated).async {
            captureSession.stopRunning()
        }
    }

    func metadataOutput(
        _ output: AVCaptureMetadataOutput,
        didOutput metadataObjects: [AVMetadataObject],
        from connection: AVCaptureConnection
    ) {
        guard !didFinish,
              let metadata = metadataObjects.first as? AVMetadataMachineReadableCodeObject,
              metadata.type == .qr,
              let value = metadata.stringValue?.trimmingCharacters(in: .whitespacesAndNewlines),
              !value.isEmpty else { return }

        didFinish = true
        captureSession?.stopRunning()
        onPayload(value)
        dismiss(animated: true)
    }

    @objc private func cancel() {
        dismiss(animated: true)
    }

    private func configureCaptureSession() {
        guard captureSession == nil,
              let device = AVCaptureDevice.default(for: .video),
              let input = try? AVCaptureDeviceInput(device: device) else {
            showErrorMessage("Camera is unavailable on this device.")
            return
        }

        let session = AVCaptureSession()
        guard session.canAddInput(input) else {
            showErrorMessage("Camera is unavailable on this device.")
            return
        }
        session.addInput(input)

        let metadataOutput = AVCaptureMetadataOutput()
        guard session.canAddOutput(metadataOutput) else {
            showErrorMessage("QR scanning is unavailable on this device.")
            return
        }
        session.addOutput(metadataOutput)
        metadataOutput.setMetadataObjectsDelegate(self, queue: .main)
        metadataOutput.metadataObjectTypes = [.qr]

        let preview = AVCaptureVideoPreviewLayer(session: session)
        preview.videoGravity = .resizeAspectFill
        preview.frame = view.bounds
        view.layer.insertSublayer(preview, at: 0)

        captureSession = session
        previewLayer = preview
        DispatchQueue.global(qos: .userInitiated).async {
            session.startRunning()
        }
    }

    private func showPermissionMessage() {
        showErrorMessage("Camera permission is required to scan a pairing code. Enable camera access in Settings.")
    }

    private func showErrorMessage(_ message: String) {
        DispatchQueue.main.async { [weak self] in
            guard let self else { return }
            let alert = UIAlertController(title: "Unable to scan QR code", message: message, preferredStyle: .alert)
            alert.addAction(UIAlertAction(title: "Close", style: .cancel) { [weak self] _ in
                self?.dismiss(animated: true)
            })
            self.present(alert, animated: true)
        }
    }
}

private final class QRScannerFrameView: UIView {
    override class var layerClass: AnyClass { CALayer.self }

    override func draw(_ rect: CGRect) {
        super.draw(rect)
        let path = UIBezierPath()
        let corner: CGFloat = 34
        let inset: CGFloat = 2
        let width = bounds.width
        let height = bounds.height

        path.move(to: CGPoint(x: inset, y: corner))
        path.addLine(to: CGPoint(x: inset, y: inset))
        path.addLine(to: CGPoint(x: corner, y: inset))
        path.move(to: CGPoint(x: width - corner, y: inset))
        path.addLine(to: CGPoint(x: width - inset, y: inset))
        path.addLine(to: CGPoint(x: width - inset, y: corner))
        path.move(to: CGPoint(x: inset, y: height - corner))
        path.addLine(to: CGPoint(x: inset, y: height - inset))
        path.addLine(to: CGPoint(x: corner, y: height - inset))
        path.move(to: CGPoint(x: width - corner, y: height - inset))
        path.addLine(to: CGPoint(x: width - inset, y: height - inset))
        path.addLine(to: CGPoint(x: width - inset, y: height - corner))

        UIColor.systemBlue.setStroke()
        path.lineWidth = 4
        path.lineCapStyle = .round
        path.stroke()
    }
}
