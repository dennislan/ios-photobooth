// iOS Camera Stream Helper (macOS)
// 通过 macOS 内置摄像头或 Continuity Camera 实现取景预览和拍照
// 使用 AVFoundation 直接访问摄像头，MJPEG 推流到本地 TCP 端口

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

// MARK: - 视频帧委托 (macOS 兼容)
class VideoDelegate: NSObject, AVCaptureVideoDataOutputSampleBufferDelegate {
    let frameCallback: (Data?) -> Void

    init(frameQueue: DispatchQueue, callback: @escaping (Data?) -> Void) {
        self.frameCallback = callback
    }

    func captureOutput(_ output: AVCaptureOutput, didOutput sampleBuffer: CMSampleBuffer, from connection: AVCaptureConnection) {
        guard let pixelBuffer = CMSampleBufferGetImageBuffer(sampleBuffer) else { return }
        let releasePixelBuffer: () -> Void = { CVPixelBufferUnlockBaseAddress(pixelBuffer, .readOnly) }
        CVPixelBufferLockBaseAddress(pixelBuffer, .readOnly)
        defer { releasePixelBuffer() }
        let ciImage = CIImage(cvPixelBuffer: pixelBuffer)
        guard let cgImage = try? CIContext(options: [:]).createCGImage(ciImage, from: ciImage.extent) else { return }
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
    // 创建保存目录
    let fileHandlerObj = FileHandler(directory: SAVE_DIR)

    // 创建 MJPEG 流媒体服务器
    let streamer = MJPEGStreamer()

    let session = AVCaptureSession()
    session.sessionPreset = .high

    // 查找摄像头设备 (优先前置/Continuity Camera)
    let discovery = AVCaptureDevice.DiscoverySession(
        deviceTypes: [.builtInWideAngleCamera],
        mediaType: .video,
        position: .unspecified
    )

    // 优先使用前置/外部摄像头
    let device = discovery.devices.first { $0.position == .front }
        ?? discovery.devices.first

    guard let device = device else {
        print("No camera found")
        exit(1)
    }

    print("Using camera: \(device.localizedName)")

    guard let videoInput = try? AVCaptureDeviceInput(device: device) else {
        print("Failed to create video input")
        exit(1)
    }

    guard session.canAddInput(videoInput) else {
        print("Cannot add video input")
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

    let videoDelegate = VideoDelegate(frameQueue: frameQueue) { frame in
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
                // Request JPEG format explicitly for reliable downstream processing
                let settings = AVCapturePhotoSettings(format: [AVVideoCodecKey: AVVideoCodecType.jpeg])
                settings.isHighResolutionPhotoEnabled = false
                photoOutput.capturePhoto(with: settings, delegate: PhotoCaptureDelegate(
                    saveDir: SAVE_DIR,
                    fileHandler: fileHandlerObj
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
