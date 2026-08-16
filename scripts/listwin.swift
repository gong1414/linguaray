// List all on-screen windows owned by LinguaRay (debug helper for the
// real-app screenshot script). Output format is parseable:
//   id=<n> layer=<l> w=<w> h=<h> x=<x> y=<y> name=<title>
// Bounds are LOGICAL points (CGWindowBounds), not pixels.
import CoreGraphics
import Foundation

let options: CGWindowListOption = [.optionOnScreenOnly]
guard let list = CGWindowListCopyWindowInfo(options, kCGNullWindowID) as? [[String: Any]] else {
    exit(1)
}
for window in list {
    let owner = window[kCGWindowOwnerName as String] as? String ?? ""
    if !owner.contains("LinguaRay") { continue }
    let layer = window[kCGWindowLayer as String] as? Int ?? -1
    let number = window[kCGWindowNumber as String] as? Int ?? -1
    let name = window[kCGWindowName as String] as? String ?? "(redacted)"
    let bounds = window[kCGWindowBounds as String] as? [String: Any] ?? [:]
    let w = bounds["Width"] as? Int ?? -1
    let h = bounds["Height"] as? Int ?? -1
    let x = bounds["X"] as? Int ?? -1
    let y = bounds["Y"] as? Int ?? -1
    print("id=\(number) layer=\(layer) w=\(w) h=\(h) x=\(x) y=\(y) name=\(name)")
}
