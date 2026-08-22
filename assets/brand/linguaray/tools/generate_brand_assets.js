const fs = require('fs');
const path = require('path');
const sharp = require('sharp');

const root = path.resolve(__dirname, '..');
const repo = path.resolve(root, '../../..');
const output = path.join(root, 'dist');
const outline = JSON.parse(
  fs.readFileSync(path.join(root, 'build/wordmark-outline.json'), 'utf8'),
);

const colors = {
  blue: '#2859D9',
  teal: '#18A6A6',
  tealOnDark: '#34C0BE',
  navy: '#13233F',
  graphite: '#172033',
  paper: '#F7F9FC',
  white: '#FFFFFF',
  black: '#000000',
};

const lPath =
  'M44 38 86 88v71c0 8 6 14 14 14h38l20 31H92c-27 0-48-21-48-48V38Z';
const rPath =
  'M91 38h65c38 0 64 23 64 57 0 25-15 44-39 52l39 57h-24l-49-66v-23h9c14 0 23-5 23-24 0-10-9-16-23-16H91V38Z';

function ensureDir(directory) {
  fs.mkdirSync(directory, { recursive: true });
}

function write(relativePath, contents) {
  const target = path.join(output, relativePath);
  ensureDir(path.dirname(target));
  fs.writeFileSync(target, contents);
  return target;
}

function svgDocument(viewBox, body, title, description) {
  return `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" viewBox="${viewBox}" role="img" aria-labelledby="title desc">
  <title id="title">${title}</title>
  <desc id="desc">${description}</desc>
${body}
</svg>
`;
}

function symbolMarkup(id, lFill, rFill) {
  return `  <g>
    <path fill="${lFill}" d="${lPath}"/>
    <path fill="${rFill}" d="${rPath}"/>
  </g>`;
}

function symbolSvg(lFill, rFill, id = 'ray-cut') {
  return svgDocument(
    '0 0 240 240',
    symbolMarkup(id, lFill, rFill),
    'LinguaRay symbol',
    'A geometric LR monogram crossed by one diagonal ray.',
  );
}

function wordmarkSvg(fill) {
  return svgDocument(
    '-3 -93 528 132',
    `  <path fill="${fill}" d="${outline.path}"/>`,
    'LinguaRay wordmark',
    'The word LinguaRay with capital L and capital R.',
  );
}

function logoSvg(dark = false) {
  const lFill = dark ? colors.white : colors.blue;
  const rFill = dark ? colors.tealOnDark : colors.teal;
  const wordFill = dark ? colors.white : colors.graphite;
  return svgDocument(
    '0 0 900 240',
    `${symbolMarkup('logo-ray-cut', lFill, rFill).replace('<g>', '<g transform="translate(20 30) scale(0.75)">')}
  <path fill="${wordFill}" transform="translate(220 153) scale(1.25)" d="${outline.path}"/>`,
    'LinguaRay logo',
    'LinguaRay horizontal logo with LR symbol and exact wordmark.',
  );
}

function appIconSvg() {
  const mark = symbolMarkup('app-ray-cut', colors.white, colors.tealOnDark).replace(
    '<g>',
    '<g transform="translate(142 176) scale(2.8)">',
  );
  return svgDocument(
    '0 0 1024 1024',
    `  <rect width="1024" height="1024" fill="${colors.navy}"/>
${mark}`,
    'LinguaRay app icon',
    'A square, unmasked app icon source for macOS and Windows.',
  );
}

function boardSvg() {
  return svgDocument(
    '0 0 1600 1100',
    `  <rect width="1600" height="1100" fill="#EEF2F7"/>
  <rect x="64" y="64" width="1472" height="972" rx="40" fill="#FFFFFF"/>
  <text x="112" y="140" font-family="Arial, sans-serif" font-size="38" font-weight="700" fill="${colors.graphite}">LinguaRay identity system</text>
  <text x="112" y="180" font-family="Arial, sans-serif" font-size="20" fill="#5A6475">LR monogram · one ray · privacy-first desktop translation</text>
  <rect x="112" y="228" width="620" height="370" rx="28" fill="${colors.navy}"/>
  <g transform="translate(248 242) scale(1.45)">${symbolMarkup('board-app-cut', colors.white, colors.tealOnDark)}</g>
  <text x="112" y="628" font-family="Arial, sans-serif" font-size="18" font-weight="700" fill="${colors.graphite}">APP ICON</text>
  <g transform="translate(990 225) scale(1.75)">${symbolMarkup('board-symbol-cut', colors.blue, colors.teal)}</g>
  <text x="990" y="628" font-family="Arial, sans-serif" font-size="18" font-weight="700" fill="${colors.graphite}">PURE SYMBOL</text>
  <g transform="translate(112 674) scale(0.48)">${symbolMarkup('board-light-cut', colors.blue, colors.teal)}</g>
  <path fill="${colors.graphite}" transform="translate(260 780) scale(0.96)" d="${outline.path}"/>
  <g transform="translate(1135 710) scale(0.075)">${symbolMarkup('board-micro-cut', colors.black, colors.black)}</g>
  <text x="1172" y="728" font-family="Arial, sans-serif" font-size="18" fill="#5A6475">18 px mono</text>
  <rect x="112" y="844" width="1376" height="142" rx="24" fill="${colors.navy}"/>
  <g transform="translate(142 853) scale(0.46)">${symbolMarkup('board-dark-cut', colors.white, colors.tealOnDark)}</g>
  <path fill="${colors.white}" transform="translate(280 947) scale(0.82)" d="${outline.path}"/>`,
    'LinguaRay brand board',
    'A presentation board showing the production-ready LinguaRay identity.',
  );
}

async function render(svg, relativePath, width, height = width) {
  const target = path.join(output, relativePath);
  ensureDir(path.dirname(target));
  await sharp(Buffer.from(svg)).resize(width, height, { fit: 'fill' }).png({ compressionLevel: 9 }).toFile(target);
  return target;
}

async function main() {
  fs.rmSync(output, { recursive: true, force: true });
  ensureDir(output);

  const symbol = symbolSvg(colors.blue, colors.teal);
  const symbolMonoBlack = symbolSvg(colors.black, colors.black, 'mono-black-cut');
  const symbolMonoWhite = symbolSvg(colors.white, colors.white, 'mono-white-cut');
  const wordmark = wordmarkSvg(colors.graphite);
  const primaryLogo = logoSvg(false);
  const readmeLight = logoSvg(false);
  const readmeDark = logoSvg(true);
  const appIcon = appIconSvg();

  write('svg/linguaray-symbol.svg', symbol);
  write('svg/linguaray-symbol-mono-black.svg', symbolMonoBlack);
  write('svg/linguaray-symbol-mono-white.svg', symbolMonoWhite);
  write('svg/linguaray-wordmark.svg', wordmark);
  write('svg/linguaray-logo-primary.svg', primaryLogo);
  write('readme/linguaray-readme-light.svg', readmeLight);
  write('readme/linguaray-readme-dark.svg', readmeDark);
  write('app-icon/linguaray-app-icon.svg', appIcon);
  write('tray/linguaray-tray-black.svg', symbolMonoBlack);
  write('tray/linguaray-tray-white.svg', symbolMonoWhite);

  await render(appIcon, 'app-icon/linguaray-app-icon-1024.png', 1024);
  await render(appIcon, 'app-icon/linguaray-app-icon-512.png', 512);
  await render(appIcon, 'app-icon/linguaray-app-icon-256.png', 256);

  const macSizes = [16, 32, 64, 128, 256, 512, 1024];
  for (const size of macSizes) {
    await render(appIcon, `macos/app_icon_${size}.png`, size);
  }

  const iconset = {
    'icon_16x16.png': 16,
    'icon_16x16@2x.png': 32,
    'icon_32x32.png': 32,
    'icon_32x32@2x.png': 64,
    'icon_128x128.png': 128,
    'icon_128x128@2x.png': 256,
    'icon_256x256.png': 256,
    'icon_256x256@2x.png': 512,
    'icon_512x512.png': 512,
    'icon_512x512@2x.png': 1024,
  };
  for (const [filename, size] of Object.entries(iconset)) {
    await render(appIcon, `macos/LinguaRay.iconset/${filename}`, size);
  }

  const windowsSizes = [16, 20, 24, 32, 40, 48, 64, 128, 256];
  for (const size of windowsSizes) {
    await render(appIcon, `windows/png/linguaray-${size}.png`, size);
  }

  for (const size of [18, 32, 36]) {
    await render(symbolMonoBlack, `tray/linguaray-tray-black-${size}.png`, size);
    await render(symbolMonoWhite, `tray/linguaray-tray-white-${size}.png`, size);
  }

  await render(readmeLight, 'readme/linguaray-readme-light.png', 1800, 480);
  await render(readmeDark, 'readme/linguaray-readme-dark.png', 1800, 480);
  await render(boardSvg(), 'preview/linguaray-brand-board.png', 1600, 1100);

  const appMac = path.join(
    repo,
    'apps/desktop/flutter/macos/Runner/Assets.xcassets/AppIcon.appiconset',
  );
  for (const size of macSizes) {
    fs.copyFileSync(
      path.join(output, `macos/app_icon_${size}.png`),
      path.join(appMac, `app_icon_${size}.png`),
    );
  }
  fs.copyFileSync(
    path.join(output, 'app-icon/linguaray-app-icon-512.png'),
    path.join(repo, 'apps/desktop/flutter/resources/images/icon.png'),
  );
  fs.copyFileSync(
    path.join(output, 'app-icon/linguaray-app-icon-256.png'),
    path.join(repo, 'apps/desktop/flutter/resources/images/app_icons/256x256.png'),
  );
  fs.copyFileSync(
    path.join(output, 'tray/linguaray-tray-white-36.png'),
    path.join(repo, 'apps/desktop/flutter/resources/images/tray_icon.png'),
  );
  fs.copyFileSync(
    path.join(output, 'tray/linguaray-tray-black-36.png'),
    path.join(repo, 'apps/desktop/flutter/resources/images/tray_icon_black.png'),
  );
  fs.copyFileSync(
    path.join(output, 'tray/linguaray-tray-black-32.png'),
    path.join(
      repo,
      'apps/desktop/flutter/macos/Runner/Assets.xcassets/StatusBarButtonImage.imageset/StatusBarButtonImage@2x.png',
    ),
  );
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
