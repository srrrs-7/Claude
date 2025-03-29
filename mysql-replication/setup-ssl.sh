#!/bin/bash
set -e

# Generate SSL certificates
echo "Generating SSL certificates..."
cd ssl
chmod +x generate-certs.sh
./generate-certs.sh
cd ..

# Copy SSL certificates to the right directories
echo "Copying SSL certificates to master..."
mkdir -p master/ssl
cp ssl/ca-cert.pem ssl/server-cert.pem ssl/server-key.pem master/ssl/

# Copy certificates to slaves
for SLAVE in slave1 slave2 slave3; do
  echo "Copying SSL certificates to $SLAVE..."
  mkdir -p $SLAVE/ssl
  cp ssl/ca-cert.pem ssl/client-cert.pem ssl/client-key.pem $SLAVE/ssl/
done

echo "SSL setup completed!"
