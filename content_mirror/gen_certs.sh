#!/bin/bash -eu

# Copyright 2024 The ChromiumOS Authors
# Use of this source code is governed by a BSD-style license that can be
# found in the LICENSE file.

OUT_DIR="$1"

# Create a random password so that after running this script, no-one can use
# this certificate authority to sign anything else.
PASS="pass:$(head -c 50 /dev/urandom | base64)"
CANAME="BazelContentMirrorLocalCA"
MYCERT="BazelContentMirrorServer"
SUBJ="/CN=${CANAME}"

rm -rf "${OUT_DIR}"
mkdir -p "${OUT_DIR}"
cd "${OUT_DIR}"

echo "Generating private key"
openssl genrsa  -passout "${PASS}" -out "${CANAME}.key" 4096

echo "Generating certificate"
openssl req -x509 -new -nodes -key "${CANAME}.key" -sha256 -days 7 \
  -out "${CANAME}.crt" -subj "${SUBJ}" -passin "${PASS}"

echo "Generating key"
openssl req -new -nodes -out "${MYCERT}.csr" -newkey rsa:4096 \
  -keyout "${MYCERT}.key" -subj "${SUBJ}"

# create a v3 ext file for SAN properties
cat > "${MYCERT}.v3.ext" << EOF
authorityKeyIdentifier=keyid,issuer
basicConstraints=CA:FALSE
keyUsage = digitalSignature, nonRepudiation, keyEncipherment, dataEncipherment
subjectAltName = @alt_names

[alt_names]
DNS.1 = localhost
IP.1 = 127.0.0.1
IP.2 = ::1
EOF

echo "Signing key"
openssl x509 -req -in "${MYCERT}.csr" -CA "${CANAME}.crt" \
  -CAkey "${CANAME}.key" -CAcreateserial -out "${MYCERT}.crt" -days 7 \
  -sha256 -extfile "${MYCERT}.v3.ext" -passin "${PASS}"

echo "Importing key"
# To ensure security (and ensure we don't require sudo), we don't modify the
# system's certificates, but instead make a copy of them and add the certificate
# to the copy.
cp /etc/ssl/certs/java/cacerts .

# The system cacerts file has an insecure known password. Change our copy's one
# and forget the new password after this script completes.
PASS="$(head -c 50 /dev/urandom | base64)"
keytool -storepasswd -new "${PASS}" -storepass changeit -keystore cacerts

keytool -importcert -file "${CANAME}.crt" -keystore cacerts \
  -alias "Bazel content mirror" -storepass "${PASS}" --noprompt
# Ensure we can't modify the keystore.
chmod 700 cacerts
