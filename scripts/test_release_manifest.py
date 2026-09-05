"""Release contract tests use ephemeral keys and synthetic installer bytes."""
import base64
import hashlib
import json
from pathlib import Path
import tempfile
import unittest

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey

from release_manifest import ASSETS, PUBLIC_KEY, ROOT, create_manifest, validate_version, write_checksums


class ReleaseManifestTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.directory = Path(self.temp.name)
        self.key = Ed25519PrivateKey.generate()
        for platform, name in ASSETS.items():
            (self.directory / name).write_bytes(f'test installer for {platform}'.encode())

    def manifest(self, **overrides):
        args = dict(directory=self.directory, tag='v0.6.1',
                    repository='gong1414/linguaray', notes='更新验证',
                    platform_signed={'windows'}, signing_seed=self.key.private_bytes_raw(),
                    public_key=self.key.public_key().public_bytes_raw())
        args.update(overrides)
        return create_manifest(**args)

    def test_signature_covers_both_platforms_and_exact_bytes(self):
        envelope = self.manifest()
        payload = base64.b64decode(envelope['payload'])
        self.key.public_key().verify(base64.b64decode(envelope['signature']), payload)
        data = json.loads(payload)
        self.assertEqual(data['version'], '0.6.1')
        self.assertEqual(data['notes'], '更新验证')
        self.assertEqual(data['repository'], 'gong1414/linguaray')
        for platform, artifact in data['artifacts'].items():
            content = (self.directory / artifact['name']).read_bytes()
            self.assertEqual(artifact['sha256'], hashlib.sha256(content).hexdigest())
            self.assertEqual(artifact['size'], len(content))
            self.assertEqual(artifact['platformSigned'], platform == 'windows')
        with self.assertRaises(InvalidSignature):
            self.key.public_key().verify(base64.b64decode(envelope['signature']),
                                         payload.replace(b'0.6.1', b'9.9.9'))

    def test_wrong_key_cannot_publish(self):
        other = Ed25519PrivateKey.generate()
        with self.assertRaisesRegex(ValueError, 'does not match'):
            self.manifest(public_key=other.public_key().public_bytes_raw())

    def test_missing_or_empty_installers_cannot_publish(self):
        path = self.directory / ASSETS['macos']
        path.write_bytes(b'')
        with self.assertRaises(ValueError):
            self.manifest()
        path.unlink()
        with self.assertRaises(FileNotFoundError):
            self.manifest()

    def test_tags_match_pubspec_and_are_stable_versions(self):
        pubspec = self.directory / 'pubspec.yaml'
        pubspec.write_text('version: 0.6.1+20\n')
        self.assertEqual(validate_version('v0.6.1', pubspec), '0.6.1')
        for tag in ['0.6.1', 'v0.6.2', 'v0.6.1-beta', 'v00.6.1', '../v0.6.1']:
            with self.subTest(tag=tag), self.assertRaises(ValueError):
                validate_version(tag, pubspec)

    def test_checksums_include_manifest_and_exclude_checksum_itself(self):
        (self.directory / 'update.json').write_text(json.dumps(self.manifest()))
        write_checksums(self.directory)
        first = (self.directory / 'SHA256SUMS.txt').read_text()
        write_checksums(self.directory)
        self.assertEqual(first, (self.directory / 'SHA256SUMS.txt').read_text())
        self.assertEqual(len(first.splitlines()), 3)
        for line in first.splitlines():
            digest, name = line.split('  ')
            self.assertEqual(digest, hashlib.sha256((self.directory / name).read_bytes()).hexdigest())

    def test_embedded_public_key_matches_release_configuration(self):
        data = json.loads(PUBLIC_KEY.read_text())
        self.assertEqual(data['algorithm'], 'Ed25519')
        self.assertEqual(len(base64.b64decode(data['publicKey'], validate=True)), 32)
        source = (ROOT / 'apps/desktop/flutter/lib/src/config/update_signing_key.dart').read_text()
        self.assertIn("'" + data['publicKey'] + "'", source)


if __name__ == '__main__':
    unittest.main()
