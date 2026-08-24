# Integrations

LinguaRay exposes one typed action table through its `linguaray://` URL scheme
and an optional loopback HTTP server. URL actions launch LinguaRay when needed;
the loopback API requires the app to be running. Actions never replace selected
text.

## URL scheme

| Link | Action |
| --- | --- |
| `linguaray://translate?text=Hello%20world` | translate supplied text |
| `linguaray://selection-translate` | read and translate the current selection |
| `linguaray://input-translate` | open a cleared input window |
| `linguaray://clipboard-translate` | translate clipboard text |
| `linguaray://capture-translate` | capture, OCR, and translate a region |
| `linguaray://capture-ocr` | capture and recognize a region |
| `linguaray://clipboard-ocr` | recognize the clipboard image |
| `linguaray://show-translation` | show the existing translation surface |
| `linguaray://show-ocr` | show the existing OCR surface |
| `linguaray://settings` | open Settings |

The decoded `text` parameter is limited to 32 KiB. Unknown actions are ignored.
User text is not logged.

## Local HTTP API

Enable the API under **Settings → Integration**. It binds only to
`127.0.0.1`; the port is shown in Settings. Open `/reference` on that address
for the generated OpenAPI reference.

Action endpoints use `POST /actions/...` and return `202 Accepted`. For
compatibility with Pot-style launchers, these routes are also available:

| Compatibility route | Canonical action |
| --- | --- |
| `POST /` or `POST /translate` with UTF-8 text | `/actions/translate` |
| `GET /config` | `/actions/settings` |
| `GET /selection_translate` | `/actions/translate-selection` |
| `GET /input_translate` | `/actions/translate-input` |
| `GET /clipboard_translate` | `/actions/translate-clipboard` |
| `GET /ocr_translate` | `/actions/capture-translate` |
| `GET /ocr_recognize` | `/actions/capture-ocr` |
| `GET /clipboard_ocr` | `/actions/clipboard-ocr` |

The API deliberately does not allow cross-origin browser access. It is an
explicitly enabled local automation surface, not a web API.

## PopClip, SnipDo, and Raycast

Source templates live under `integrations/`. Release packaging adds the
canonical LinguaRay icon and produces installable PopClip and SnipDo archives.
All three integrations call the URL scheme, so they do not depend on a fixed
HTTP port or require the local API to be enabled.

- PopClip passes its already percent-encoded selected text to the URL scheme.
- SnipDo percent-encodes `PLAIN_TEXT` before starting the registered URL.
- Raycast offers selection, clipboard, input, and screenshot OCR commands.

Each release publishes `LinguaRay-PopClip.popclipext.zip`,
`LinguaRay-SnipDo.zip`, and `LinguaRay-Raycast-source.zip`. The PopClip archive
can be unzipped and opened directly. SnipDo imports the JSON and PowerShell
action from its extracted folder. The Raycast archive is source for local
development until the extension is accepted into the Raycast Store.

The signed Windows installer registers `linguaray://` for the current user and
removes it on uninstall. macOS registers the same scheme from the signed app
bundle.
