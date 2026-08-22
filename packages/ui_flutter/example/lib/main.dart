import 'package:linguaray_ui/linguaray_ui.dart';
import 'package:fluentui_system_icons/fluentui_system_icons.dart';
import 'package:flutter/widgets.dart';
part 'atoms_sections.part.dart';

void main() => runApp(const GalleryApp());

class GalleryApp extends StatelessWidget {
  const GalleryApp({super.key});

  @override
  Widget build(BuildContext context) => WidgetsApp(
        title: 'LinguaRay UI',
        color: const Color(0xFF6B4DFF),
        pageRouteBuilder: <T>(RouteSettings settings, WidgetBuilder builder) =>
            PageRouteBuilder<T>(
          settings: settings,
          pageBuilder: (context, _, __) => builder(context),
        ),
        home: const Gallery(),
      );
}

class Gallery extends StatefulWidget {
  const Gallery({
    super.key,
    this.initialTheme = DesignThemeName.studioLight,
    this.typography,
  });

  final DesignThemeName initialTheme;

  /// Swaps the type roles onto other faces. The specimen test uses this to
  /// bind the real macOS faces, which `flutter test` does not load by default.
  final DesignTypography? typography;

  @override
  State<Gallery> createState() => _GalleryState();
}

class _GalleryState extends State<Gallery> {
  late DesignThemeName _theme = widget.initialTheme;

  DesignTokens get _tokens {
    final base = _theme.tokens;
    if (widget.typography == null) return base;
    return DesignTokens(
      brightness: base.brightness,
      colors: base.colors,
      radii: base.radii,
      metrics: base.metrics,
      shadows: base.shadows,
      backdrop: base.backdrop,
      progressGradient: base.progressGradient,
      typography: widget.typography!,
    );
  }

  @override
  Widget build(BuildContext context) => DesignThemeProvider(
        theme: _theme,
        tokens: _tokens,
        child: Builder(
          builder: (context) => ColoredBox(
            color: context.colors.canvas,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                _ThemeBar(
                  value: _theme,
                  onChanged: (theme) => setState(() => _theme = theme),
                ),
                const Expanded(child: _Atoms()),
              ],
            ),
          ),
        ),
      );
}

class _ThemeBar extends StatelessWidget {
  const _ThemeBar({required this.value, required this.onChanged});

  final DesignThemeName value;
  final ValueChanged<DesignThemeName> onChanged;

  @override
  Widget build(BuildContext context) {
    final tokens = context.tokens;
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 14),
      decoration: BoxDecoration(
        color: tokens.colors.chrome,
        border: Border(
          bottom: BorderSide(
            color: tokens.colors.hairline,
            width: context.hairlineWidth,
          ),
        ),
      ),
      child: Row(
        children: [
          Text(
            'LinguaRay UI · Flutter',
            style: tokens.typography.displayStyle(
              fontSize: 13,
              fontWeight: FontWeight.w700,
              height: 1,
              color: tokens.colors.fg,
            ),
          ),
          const Spacer(),
          SegmentedControl<DesignThemeName>(
            value: value,
            onChanged: onChanged,
            items: [
              for (final theme in DesignThemeName.values)
                SegmentedItem(
                  value: theme,
                  label: Text(designThemeMeta[theme]!.title),
                ),
            ],
          ),
        ],
      ),
    );
  }
}

class _Atoms extends StatefulWidget {
  const _Atoms();

  @override
  State<_Atoms> createState() => _AtomsState();
}

class _Section extends StatelessWidget {
  const _Section({required this.title, required this.children});

  final String title;
  final List<Widget> children;

  @override
  Widget build(BuildContext context) {
    final tokens = context.tokens;
    return Padding(
      padding: const EdgeInsets.only(bottom: 36),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            title,
            style: tokens.typography.displayStyle(
              fontSize: 15,
              fontWeight: FontWeight.w700,
              height: 1,
              color: tokens.colors.fg,
            ),
          ),
          const SizedBox(height: 6),
          const Divider(),
          const SizedBox(height: 16),
          for (final child in children)
            Padding(
              padding: const EdgeInsets.only(bottom: 16),
              child: child,
            ),
        ],
      ),
    );
  }
}

/// A wrapping row, so a strip of atoms reflows instead of overflowing.
class _Row extends StatelessWidget {
  const _Row(this.children);

  final List<Widget> children;

  @override
  Widget build(BuildContext context) => Wrap(
        spacing: 12,
        runSpacing: 12,
        crossAxisAlignment: WrapCrossAlignment.center,
        children: children,
      );
}
