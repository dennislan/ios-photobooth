// 相机辅助工具 (macOS)
// 使用 AVFoundation 访问 iPhone (Continuity Camera) 或内置摄像头
// 提供 MJPEG 实时预览流 + JPEG 拍照捕获
// 由 Rust 后端 (stream.rs) 作为子进程启动，通过 stdin/stdout 通信

import Foundation
import AVFoundation
import CoreVideo
import CoreGraphics
import ImageIO
import CoreImage

// MARK: - 配置
let WEBSOCKET_PORT = 27183
let SAVE_DIR = FileManager.default.temporaryDirectory.appendingPathComponent("photobooth")

// MARK: - 文件处理器
class FileHandler {
    let saveDirectory: URL

    init(directory: URL) {
        self.saveDirectory = directory
        try? FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
    }

    func savePhotoData(_ data: Data) -> String? {
        let fileName = "photo_\(UUID().uuidString).jpg"
        let fileURL = saveDirectory.appendingPathComponent(fileName)
        do {
            try data.write(to: fileURL)
            return fileURL.path
        } catch {
            fputs("Error saving photo: \(error)\n", stderr)
            return nil
        }
    }
}

// MARK: - MJPEG TCP 服务器
class MJPEGStreamer {
    private var serverSocket: Int32 = -1
    private var clientSocket: Int32 = -1
    private var isRunning = false

    func start(port: Int, frameProvider: @escaping () -> Data?) {
        guard port > 0 else { return }

        serverSocket = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP)
        guard serverSocket >= 0 else { return }

        var addr = sockaddr_in()
        addr.sin_len = UInt8(MemoryLayout.size(ofValue: addr))
        addr.sin_family = sa_family_t(AF_INET)
        addr.sin_addr.s_addr = inet_addr("127.0.0.1")
        addr.sin_port = UInt16(port).bigEndian

        var addrLen = socklen_t(MemoryLayout.size(ofValue: addr))
        withUnsafePointer(to: &addr) { ptr in
            ptr.withMemoryRebound(to: sockaddr.self, capacity: 1) {
                bind(serverSocket, $0, addrLen)
            }
        }

        listen(serverSocket, 5)
        isRunning = true

        Thread {
            while self.isRunning {
                var clientAddr = sockaddr_in()
                var clientAddrLen = socklen_t(MemoryLayout.size(ofValue: clientAddr))
                var clientAddrPtr = clientAddr
                let client = accept(self.serverSocket,
                    UnsafeMutableRawPointer(&clientAddr).bindMemory(to: sockaddr.self, capacity: 1),
                    &clientAddrLen)
                guard client >= 0 else { continue }
                self.clientSocket = client

                let header = "HTTP/1.1 200 OK\r\nContent-Type: multipart/x-mixed-replace; boundary=frame\r\n\r\n"
                guard let headerData = header.data(using: .utf8) else { continue }
                var sent = send(client, (headerData as NSData).bytes.bindMemory(to: UInt8.self, capacity: headerData.count), headerData.count, 0)
                if sent < headerData.count {
                    close(client); self.clientSocket = -1; continue
                }

                while self.isRunning && self.clientSocket >= 0 {
                    guard let frame = frameProvider(), !frame.isEmpty else {
                        usleep(1000)
                        continue
                    }

                    let boundary = "\r\n--frame\r\nContent-Type: image/jpeg\r\nContent-Length: \(frame.count)\r\n\r\n"
                    guard let boundaryData = boundary.data(using: .utf8) else { continue }

                    let total = boundaryData.count + frame.count
                    var buf = [UInt8](repeating: 0, count: total)
                    buf.replaceSubrange(0..<boundaryData.count, with: [UInt8](boundaryData))
                    buf.replaceSubrange(boundaryData.count..<(boundaryData.count + frame.count), with: [UInt8](frame))

                    sent = send(client, buf, buf.count, 0)
                    if sent < 0 { break }
                }

                shutdown(client, SHUT_RDWR)
                close(client)
                self.clientSocket = -1
            }
        }.start()
    }

    func stop() {
        isRunning = false
        if clientSocket >= 0 {
            shutdown(clientSocket, SHUT_RDWR)
            close(clientSocket)
            clientSocket = -1
        }
        if serverSocket >= 0 {
            close(serverSocket)
            serverSocket = -1
        }
    }
}

// MARK: - 视频帧委托
class VideoDelegate: NSObject, AVCaptureVideoDataOutputSampleBufferDelegate {
    let frameCallback: (Data?) -> Void

    init(callback: @escaping (Data?) -> Void) {
        self.frameCallback = callback
    }

    func captureOutput(_ output: AVCaptureOutput, didOutput sampleBuffer: CMSampleBuffer, from connection: AVCaptureConnection) {
        guard let pixelBuffer = CMSampleBufferGetImageBuffer(sampleBuffer) else { return }
        let releasePixelBuffer: () -> Void = { CVPixelBufferUnlockBaseAddress(pixelBuffer, .readOnly) }
        CVPixelBufferLockBaseAddress(pixelBuffer, .readOnly)
        defer { releasePixelBuffer() }
        let ciImage = CIImage(cvPixelBuffer: pixelBuffer)
        guard let cgImage = CIContext(options: [:]).createCGImage(ciImage, from: ciImage.extent) else { return }
        if let jpeg = cgImage.jpegData(compressionQuality: 0.7) {
            frameCallback(jpeg)
        }
    }
}

// MARK: - CGImage → JPEG
extension CGImage {
    func jpegData(compressionQuality: CGFloat) -> Data? {
        let mutableData = CFDataCreateMutable(nil, 0)
        guard let destination = CGImageDestinationCreateWithData(
            mutableData!, kUTTypeJPEG as CFString, 1, nil
        ) else { return nil }

        CGImageDestinationAddImage(destination, self, [
            kCGImageDestinationLossyCompressionQuality: compressionQuality
        ] as CFDictionary)

        guard CGImageDestinationFinalize(destination) else { return nil }

        let length = CFDataGetLength(mutableData!)
        let bytes = CFDataGetBytePtr(mutableData!)
        return Data(bytes: bytes!, count: length)
    }
}

// MARK: - 照片捕获代理
@available(macOS 10.15, *)
class PhotoCaptureDelegate: NSObject, AVCapturePhotoCaptureDelegate {
    let saveDir: URL
    let fileHandler: FileHandler

    init(saveDir: URL, fileHandler: FileHandler) {
        self.saveDir = saveDir
        self.fileHandler = fileHandler
    }

    func photoOutput(_ output: AVCapturePhotoOutput, didFinishProcessingPhoto photo: AVCapturePhoto, error: Error?) {
        if let error = error {
            fputs("Photo capture error: \(error)\n", stderr)
            return
        }
        if let data = photo.fileDataRepresentation() {
            if let path = fileHandler.savePhotoData(data) {
                // Signal to the parent Rust process which file was saved
                print("CAPTURE_SAVED:\(path)")
                fflush(stdout)
            }
        }
    }
}

// MARK: - 主逻辑
@available(macOS 10.15, *)
func main() {
    // 版本标识 — Rust 端据此检测 helper 版本，拒绝旧版二进制
    print("HELPER_V2")
    fflush(stdout)

    // 从命令行参数获取设备 ID 和设备名称（均可选）
    // 有 device_id → iPhone 模式（仅使用 Continuity Camera / 外部设备）
    // 无 device_id → 内置摄像头模式（直接使用 Mac 前置摄像头）
    //
    // 参数约定（由 Rust stream.rs 传入）：
    //   argv[1] = device_id   (libimobiledevice UDID，用于区分有无 iPhone)
    //   argv[2] = device_name (ideviceinfo -k DeviceName，用于匹配 AVFoundation 设备)
    let cmdArgs = CommandLine.arguments.dropFirst()
    let targetDeviceId: String? = cmdArgs.first
    let targetDeviceName: String? = cmdArgs.dropFirst().first

    // 创建保存目录
    let fileHandlerObj = FileHandler(directory: SAVE_DIR)

    // 创建 MJPEG 流媒体服务器
    let streamer = MJPEGStreamer()

    let session = AVCaptureSession()
    session.sessionPreset = .high

    // 请求相机授权（helper 继承父 .app 的 TCC 权限，此处做防御性检查）
    let authStatus = AVCaptureDevice.authorizationStatus(for: .video)
    if authStatus == .denied || authStatus == .restricted {
        print("ERR:相机权限被拒绝。请在「系统设置 > 隐私与安全 > 相机」中允许大头贴，然后重启应用。")
        fflush(stdout)
        Thread.sleep(forTimeInterval: 0.3)
        exit(1)
    }
    if authStatus == .notDetermined {
        AVCaptureDevice.requestAccess(for: .video) { _ in }
        Thread.sleep(forTimeInterval: 2.0)
    }

    // ── 模式 A：iPhone 模式（targetDeviceId 存在）──
    if targetDeviceId != nil {
        // 使用 .external 类型发现 Continuity Camera (iPhone over USB/WiFi)
        // 同时加入 .continuityCamera（macOS 15+）以覆盖更多场景
        var deviceTypes: [AVCaptureDevice.DeviceType] = []
        if #available(macOS 14.0, *) {
            deviceTypes.append(.external)
        }
        if #available(macOS 15.0, *) {
            deviceTypes.append(.continuityCamera)
        }

        let discovery = AVCaptureDevice.DiscoverySession(
            deviceTypes: deviceTypes,
            mediaType: .video,
            position: .unspecified
        )

        // 期望匹配的设备名称（来自 ideviceinfo -k DeviceName）。
        // Continuity Camera 在 AVFoundation 中的 localizedName 通常为 "{DeviceName} Camera"。
        let expectedName = targetDeviceName ?? ""

        // 查找匹配的 Continuity Camera 设备。
        // 如果有 expectedName，按名称匹配；否则退化为取第一个。
        var captureDevice: AVCaptureDevice? = nil
        var discoveredNames: [String] = []
        for _ in 0..<15 {
            let devices = discovery.devices
            if !devices.isEmpty {
                discoveredNames = devices.map { $0.localizedName }
                if expectedName.isEmpty {
                    captureDevice = devices.first
                } else {
                    // 按设备名匹配：Continuity Camera 的 localizedName 为 "{DeviceName} Camera"
                    captureDevice = devices.first { device in
                        let name = device.localizedName
                        return name == "\(expectedName) Camera"
                            || name == expectedName
                            || name.hasPrefix("\(expectedName)")
                    }
                }
            }
            if captureDevice != nil { break }
            Thread.sleep(forTimeInterval: 0.5)
        }

        guard let device = captureDevice else {
            let foundList = discoveredNames.isEmpty
                ? "无"
                : discoveredNames.joined(separator: ", ")
            print("ERR:未检测到匹配的 iPhone 相机。期望设备「\(expectedName)」，AVFoundation 发现: \(foundList)。请确认：1) iPhone 已解锁并亮屏 2) 系统设置 > 隐私与安全 > 相机 已授权本应用 3) iPhone 上：控制中心 > 屏幕镜像 > 选择本机开启连续互通相机")
            fflush(stdout)
            Thread.sleep(forTimeInterval: 0.3)
            exit(1)
        }

        fputs("Using iPhone camera (Continuity): \(device.localizedName) (expected: \(expectedName))\n", stderr)

        // 向 Rust 报告实际选中的设备名，供其验证是否与期望设备匹配
        print("DEVICE_SELECTED:\(device.localizedName)")
        fflush(stdout)

        // 后续初始化与之前一致
        trySetupAndStream(session: session, device: device, streamer: streamer, fileHandler: fileHandlerObj)
    }
    // ── 模式 B：内置摄像头（无 device_id）──
    else {
        let discovery = AVCaptureDevice.DiscoverySession(
            deviceTypes: [.builtInWideAngleCamera],
            mediaType: .video,
            position: .front
        )

        guard let device = discovery.devices.first else {
            print("ERR:未发现内置摄像头。")
            fflush(stdout)
            Thread.sleep(forTimeInterval: 0.3)
            exit(1)
        }

        fputs("Using built-in camera: \(device.localizedName)\n", stderr)

        // 向 Rust 报告实际选中的设备名
        print("DEVICE_SELECTED:\(device.localizedName)")
        fflush(stdout)

        trySetupAndStream(session: session, device: device, streamer: streamer, fileHandler: fileHandlerObj)
    }
}

// MARK: - 通用初始化与流启动
@available(macOS 10.15, *)
func trySetupAndStream(session: AVCaptureSession, device: AVCaptureDevice, streamer: MJPEGStreamer, fileHandler: FileHandler) {
    guard let videoInput = try? AVCaptureDeviceInput(device: device) else {
        print("ERR:无法创建视频输入，摄像头可能被其他应用占用。")
        fflush(stdout)
        Thread.sleep(forTimeInterval: 0.3)
        exit(1)
    }

    guard session.canAddInput(videoInput) else {
        print("ERR:无法添加视频输入到捕获会话。")
        fflush(stdout)
        Thread.sleep(forTimeInterval: 0.3)
        exit(1)
    }
    session.addInput(videoInput)

    let photoOutput = AVCapturePhotoOutput()
    if session.canAddOutput(photoOutput) { session.addOutput(photoOutput) }

    let videoOutput = AVCaptureVideoDataOutput()
    videoOutput.alwaysDiscardsLateVideoFrames = true
    if session.canAddOutput(videoOutput) { session.addOutput(videoOutput) }

    var lastFrame: Data?
    let frameQueue = DispatchQueue(label: "photobooth.frame.queue")

    let videoDelegate = VideoDelegate { frame in
        frameQueue.sync { lastFrame = frame }
    }
    videoOutput.setSampleBufferDelegate(videoDelegate, queue: frameQueue)

    streamer.start(port: WEBSOCKET_PORT) {
        frameQueue.sync { lastFrame }
    }

    session.startRunning()
    print("STREAM_READY")
    fflush(stdout)
    fputs("Camera session started. Streaming on port \(WEBSOCKET_PORT)\n", stderr)

    // 监听 stdin 用于拍照命令
    while let line = readLine() {
        let command = line.trimmingCharacters(in: .whitespacesAndNewlines)

        if command == "capture" {
            if #available(macOS 10.15, *) {
                let settings = AVCapturePhotoSettings(format: [AVVideoCodecKey: AVVideoCodecType.jpeg])
                settings.isHighResolutionPhotoEnabled = false
                photoOutput.capturePhoto(with: settings, delegate: PhotoCaptureDelegate(
                    saveDir: SAVE_DIR,
                    fileHandler: fileHandler
                ))
            }
        } else if command == "quit" {
            break
        }
    }

    session.stopRunning()
    streamer.stop()
    print("Camera stream stopped")
}

if #available(macOS 10.15, *) {
    main()
}
