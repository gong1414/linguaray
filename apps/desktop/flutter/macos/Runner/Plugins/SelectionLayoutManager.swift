import AppKit

/// Keeps Flutter's requested selection tint when an embedded text view is not
/// the first responder. AppKit otherwise substitutes its inactive selection
/// colour for platform views.
final class SelectionLayoutManager: NSLayoutManager {
  var selectionColor: NSColor?

  override func fillBackgroundRectArray(
    _ rectArray: UnsafePointer<NSRect>,
    count rectCount: Int,
    forCharacterRange charRange: NSRange,
    color: NSColor
  ) {
    let resolvedColor = selectionColor ?? color
    super.fillBackgroundRectArray(
      rectArray,
      count: rectCount,
      forCharacterRange: charRange,
      color: resolvedColor
    )
  }
}
