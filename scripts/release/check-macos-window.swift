import Foundation
import CoreGraphics
import ImageIO

// Inspect only the app window's interior: desktop/menu pixels cannot pass this check.
let pid = Int(CommandLine.arguments[1])!
let output = CommandLine.arguments[2]
let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []
var result: [String: Any] = ["visible": false, "nonBlank": false]
for window in windows {
    guard let owner = window[kCGWindowOwnerPID as String] as? Int, owner == pid,
          let bounds = window[kCGWindowBounds as String] as? [String: Any],
          let width = bounds["Width"] as? Double, let height = bounds["Height"] as? Double,
          width > 300, height > 200,
          let identifier = window[kCGWindowNumber as String] as? Int else { continue }
    let capture = Process()
    capture.executableURL = URL(fileURLWithPath: "/usr/sbin/screencapture")
    capture.arguments = ["-x", "-l", String(identifier), output]
    try capture.run(); capture.waitUntilExit()
    guard capture.terminationStatus == 0,
          let source = CGImageSourceCreateWithURL(URL(fileURLWithPath: output) as CFURL, nil),
          let image = CGImageSourceCreateImageAtIndex(source, 0, nil) else { continue }
    let w = image.width, h = image.height
    var pixels = [UInt8](repeating: 0, count: w * h * 4)
    let metrics: (Double, Double) = pixels.withUnsafeMutableBytes { storage in
        guard let context = CGContext(data: storage.baseAddress, width: w, height: h,
            bitsPerComponent: 8, bytesPerRow: w * 4, space: CGColorSpaceCreateDeviceRGB(),
            bitmapInfo: CGBitmapInfo.byteOrder32Big.rawValue | CGImageAlphaInfo.premultipliedLast.rawValue) else { return (0, 0) }
        context.draw(image, in: CGRect(x: 0, y: 0, width: CGFloat(w), height: CGFloat(h)))
        let bytes = storage.bindMemory(to: UInt8.self)
        var values = [Double]()
        for y in stride(from: 80, to: h - 80, by: 4) {
            for x in stride(from: 40, to: w - 40, by: 4) {
                let index = (y * w + x) * 4
                values.append((Double(bytes[index]) + Double(bytes[index + 1]) + Double(bytes[index + 2])) / 3)
            }
        }
        guard !values.isEmpty else { return (0, 0) }
        let mean = values.reduce(0, +) / Double(values.count)
        let variance = values.reduce(0) { $0 + ($1 - mean) * ($1 - mean) } / Double(values.count)
        return ((values.max() ?? 0) - (values.min() ?? 0), sqrt(variance))
    }
    result = ["visible": true, "windowId": identifier, "pixelRange": metrics.0,
              "pixelDeviation": metrics.1, "nonBlank": metrics.0 > 30 && metrics.1 > 3]
    break
}
let data = try JSONSerialization.data(withJSONObject: result, options: [.sortedKeys])
print(String(data: data, encoding: .utf8)!)
