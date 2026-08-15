// Print the CGWindowID of LinguaRay's visible (layer-0) window.
// Owner-name lookup works without Screen Recording permission; the calling
// terminal still needs that permission for `screencapture -l` to capture
// window CONTENTS (otherwise the shot is wallpaper-only).
import CoreGraphics
import Foundation

let options: CGWindowListOption = [.optionOnScreenOnly]
guard let list = CGWindowListCopyWindowInfo(options, kCGNullWindowID) as? [[String: Any]] else {
    exit(1)
}
for window in list {
    let owner = window[kCGWindowOwnerName as String] as? String ?? ""
    let layer = window[kCGWindowLayer as String] as? Int ?? -1
    if owner.contains("LinguaRay") && layer == 0 {
        if let number = window[kCGWindowNumber as String] as? Int {
            print(number)
            exit(0)
        }
    }
}
exit(1)
