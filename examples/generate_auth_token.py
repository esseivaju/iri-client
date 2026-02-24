#!/usr/bin/env python3
"""Generate an OAuth2 token using authlib OAuth2Session + private_key_jwt.

Install prerequisites once:
  pip install authlib requests

Defaults:
- client id: .auth/clientid.txt
- private key: .auth/priv_key.pem
- token endpoint: https://oidc.nersc.gov/c2id/token
"""

from __future__ import annotations

import argparse
import json
from pathlib import Path

from authlib.integrations.requests_client import OAuth2Session
from authlib.oauth2.rfc7523 import PrivateKeyJWT


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--client-id-file", default=".auth/clientid.txt")
    parser.add_argument("--private-key-file", default=".auth/priv_key.pem")
    parser.add_argument("--token-url", default="https://oidc.nersc.gov/c2id/token")
    parser.add_argument("--scope", default=None, help="Optional scope string")
    parser.add_argument("--show-response", action="store_true")
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    client_id = Path(args.client_id_file).read_text(encoding="utf-8").strip()
    private_key = Path(args.private_key_file).read_text(encoding="utf-8")

    # Use the Authlib/NERSC pattern for private_key_jwt client authentication.
    oauth = OAuth2Session(
        client_id,
        private_key,
        PrivateKeyJWT(args.token_url),
        grant_type="client_credentials",
        token_endpoint=args.token_url,
    )

    fetch_kwargs: dict[str, str] = {}
    if args.scope:
        fetch_kwargs["scope"] = args.scope
    token = oauth.fetch_token(**fetch_kwargs)

    if args.show_response:
        print(json.dumps(token, indent=2, sort_keys=True))
    else:
        print(token["access_token"])
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
