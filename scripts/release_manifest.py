#!/usr/bin/env python3
"""Validate release versions and sign the update manifest with Ed25519.

The signing seed is read only from LINGUARAY_UPDATE_SIGNING_KEY. Neither this
script nor its errors print it. The public key is committed with the client.
"""
from __future__ import annotations

import argparse
import base64
import hashlib
import json
import os
from pathlib import Path
import re
import sys
from datetime import datetime, timezone

ROOT = Path(__file__).resolve().parents[1]
PUBLIC_KEY = ROOT / 'assets/update/public-key.json'
ASSETS = {'macos': 'LinguaRay-macos.dmg', 'windows': 'LinguaRay-windows-x64.exe'}
VERSION = re.compile(r'(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)')


def version_from_tag(tag: str) -> str:
    value = tag.removeprefix('v')
    if not VERSION.fullmatch(value) or tag != f'v{value}':
        raise ValueError('Release tags must be stable semantic versions: vMAJOR.MINOR.PATCH')
    return value


def validate_version(tag: str, pubspec: Path) -> str:
    version = version_from_tag(tag)
    match = re.search(r'^version:\s*(\S+)\s*$', pubspec.read_text(), re.MULTILINE)
    if not match or match.group(1).split('+')[0] != version:
        raise ValueError('Release tag does not match the desktop pubspec version')
    return version


def file_sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open('rb') as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b''):
            digest.update(chunk)
    return digest.hexdigest()


def create_manifest(directory: Path, tag: str, repository: str, notes: str,
                    platform_signed: set[str], signing_seed: bytes, public_key: bytes) -> dict:
    from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey
    from cryptography.hazmat.primitives import serialization

    version = version_from_tag(tag)
    if not re.fullmatch(r'[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+', repository):
        raise ValueError('Invalid repository name')
    private = Ed25519PrivateKey.from_private_bytes(signing_seed)
    derived = private.public_key().public_bytes(
        serialization.Encoding.Raw, serialization.PublicFormat.Raw)
    if derived != public_key:
        raise ValueError('Signing key does not match the public key embedded in the client')
    artifacts = {}
    for platform, name in ASSETS.items():
        path = directory / name
        size = path.stat().st_size
        if not 0 < size <= 1024 * 1024 * 1024:
            raise ValueError(f'Invalid release artifact size: {name}')
        digest = file_sha256(path)
        artifacts[platform] = {
            'name': name, 'sha256': digest, 'size': size,
            'platformSigned': platform in platform_signed,
        }
    payload = json.dumps({
        'schemaVersion': 1, 'version': version, 'repository': repository,
        'publishedAt': datetime.now(timezone.utc).isoformat(),
        'notes': notes, 'artifacts': artifacts,
    }, ensure_ascii=False, separators=(',', ':'), sort_keys=True).encode()
    return {'payload': base64.b64encode(payload).decode(),
            'signature': base64.b64encode(private.sign(payload)).decode()}


def write_checksums(directory: Path) -> None:
    lines = []
    for path in sorted(directory.iterdir()):
        if not path.is_file() or path.name == 'SHA256SUMS.txt':
            continue
        digest = file_sha256(path)
        lines.append(f'{digest}  {path.name}\n')
    (directory / 'SHA256SUMS.txt').write_text(''.join(lines))


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--tag', required=True)
    parser.add_argument('--artifacts', type=Path)
    parser.add_argument('--repository', default='gong1414/linguaray')
    parser.add_argument('--notes-file', type=Path)
    parser.add_argument('--platform-signed', nargs='*', choices=ASSETS, default=[])
    args = parser.parse_args()
    version = validate_version(args.tag, ROOT / 'apps/desktop/flutter/pubspec.yaml')
    if args.artifacts is None:
        print(f'Release version verified: {version}')
        return
    try:
        seed = base64.b64decode(os.environ['LINGUARAY_UPDATE_SIGNING_KEY'], validate=True)
        public = base64.b64decode(json.loads(PUBLIC_KEY.read_text())['publicKey'], validate=True)
    except (KeyError, ValueError):
        raise ValueError('Update signing key is missing or malformed') from None
    notes = args.notes_file.read_text() if args.notes_file else f'LinguaRay {version}'
    envelope = create_manifest(args.artifacts, args.tag, args.repository, notes,
                               set(args.platform_signed), seed, public)
    (args.artifacts / 'update.json').write_text(json.dumps(envelope, indent=2) + '\n')
    write_checksums(args.artifacts)
    print('Signed update.json and SHA256SUMS.txt created for both desktop platforms.')


if __name__ == '__main__':
    try:
        main()
    except (ValueError, OSError) as error:
        print(f'Release validation failed: {error}', file=sys.stderr)
        sys.exit(1)
