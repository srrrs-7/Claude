#!/bin/bash
set -e

# Create CA certificate
openssl genrsa 2048 > ca-key.pem
openssl req -new -x509 -nodes -days 3600 \
    -key ca-key.pem -out ca-cert.pem \
    -subj "/C=JP/ST=Tokyo/L=Tokyo/O=MySQL/OU=Replication/CN=MySQL_CA"

# Create server certificate
openssl req -newkey rsa:2048 -days 3600 -nodes \
    -keyout server-key.pem -out server-req.pem \
    -subj "/C=JP/ST=Tokyo/L=Tokyo/O=MySQL/OU=Server/CN=mysql-master"
openssl rsa -in server-key.pem -out server-key.pem
openssl x509 -req -in server-req.pem -days 3600 \
    -CA ca-cert.pem -CAkey ca-key.pem -set_serial 01 -out server-cert.pem

# Create client certificate
openssl req -newkey rsa:2048 -days 3600 -nodes \
    -keyout client-key.pem -out client-req.pem \
    -subj "/C=JP/ST=Tokyo/L=Tokyo/O=MySQL/OU=Client/CN=repl"
openssl rsa -in client-key.pem -out client-key.pem
openssl x509 -req -in client-req.pem -days 3600 \
    -CA ca-cert.pem -CAkey ca-key.pem -set_serial 02 -out client-cert.pem

# Verify certificates
openssl verify -CAfile ca-cert.pem server-cert.pem client-cert.pem

# Fix permissions
chmod 644 *.pem

echo "SSL certificates generated successfully!"
