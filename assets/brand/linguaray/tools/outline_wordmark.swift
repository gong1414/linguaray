import CoreGraphics
import CoreText
import Foundation

struct Outline: Codable {
    let path: String
    let minX: Double
    let minY: Double
    let width: Double
    let height: Double
}

func number(_ value: CGFloat) -> String {
    let rounded = (value * 100).rounded() / 100
    if rounded == -0 { return "0" }
    return String(format: "%.2f", rounded)
}

func pathData(_ path: CGPath, offset: CGPoint) -> String {
    var commands: [String] = []
    path.applyWithBlock { pointer in
        let element = pointer.pointee
        func point(_ index: Int) -> CGPoint {
            let original = element.points[index]
            return CGPoint(x: original.x + offset.x, y: -(original.y + offset.y))
        }

        switch element.type {
        case .moveToPoint:
            let p = point(0)
            commands.append("M\(number(p.x)) \(number(p.y))")
        case .addLineToPoint:
            let p = point(0)
            commands.append("L\(number(p.x)) \(number(p.y))")
        case .addQuadCurveToPoint:
            let c = point(0)
            let p = point(1)
            commands.append("Q\(number(c.x)) \(number(c.y)) \(number(p.x)) \(number(p.y))")
        case .addCurveToPoint:
            let c1 = point(0)
            let c2 = point(1)
            let p = point(2)
            commands.append("C\(number(c1.x)) \(number(c1.y)) \(number(c2.x)) \(number(c2.y)) \(number(p.x)) \(number(p.y))")
        case .closeSubpath:
            commands.append("Z")
        @unknown default:
            break
        }
    }
    return commands.joined(separator: " ")
}

let arguments = CommandLine.arguments
guard arguments.count == 3 else {
    fputs("usage: outline_wordmark <font-file> <output-json>\n", stderr)
    exit(2)
}

let fontURL = URL(fileURLWithPath: arguments[1])
let outputURL = URL(fileURLWithPath: arguments[2])
guard
    let provider = CGDataProvider(url: fontURL as CFURL),
    let graphicsFont = CGFont(provider)
else {
    fputs("unable to load font\n", stderr)
    exit(3)
}

let fontSize: CGFloat = 100
let font = CTFontCreateWithGraphicsFont(graphicsFont, fontSize, nil, nil)
let attributes: [NSAttributedString.Key: Any] = [
    NSAttributedString.Key(kCTFontAttributeName as String): font,
    NSAttributedString.Key(kCTKernAttributeName as String): -1.0,
]
let attributed = NSAttributedString(string: "LinguaRay", attributes: attributes)
let line = CTLineCreateWithAttributedString(attributed)
let lineBounds = CTLineGetBoundsWithOptions(line, [.useGlyphPathBounds])

var paths: [String] = []
let runs = CTLineGetGlyphRuns(line) as! [CTRun]
for run in runs {
    let count = CTRunGetGlyphCount(run)
    var glyphs = [CGGlyph](repeating: 0, count: count)
    var positions = [CGPoint](repeating: .zero, count: count)
    CTRunGetGlyphs(run, CFRange(location: 0, length: 0), &glyphs)
    CTRunGetPositions(run, CFRange(location: 0, length: 0), &positions)

    for index in 0..<count {
        guard let glyphPath = CTFontCreatePathForGlyph(font, glyphs[index], nil) else { continue }
        paths.append(pathData(glyphPath, offset: positions[index]))
    }
}

let outline = Outline(
    path: paths.joined(separator: " "),
    minX: Double(lineBounds.minX),
    minY: Double(-lineBounds.maxY),
    width: Double(lineBounds.width),
    height: Double(lineBounds.height)
)
let encoder = JSONEncoder()
encoder.outputFormatting = [.prettyPrinted, .sortedKeys]
let data = try encoder.encode(outline)
try data.write(to: outputURL, options: .atomic)
