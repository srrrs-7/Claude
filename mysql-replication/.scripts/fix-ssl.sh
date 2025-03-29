#!/bin/bash
set -e

echo "===== SSL証明書の問題を修正します ====="

# 現在のディレクトリを取得
ROOT_DIR=$(pwd)
echo "Working in directory: $ROOT_DIR"

# 既存の証明書ファイルを探して表示
echo "Looking for existing certificate files:"
find "$ROOT_DIR" -name "*.pem" -type f

# 既存の証明書ファイルを削除
echo "Cleaning up existing certificates..."
find "$ROOT_DIR" -name "*.pem" -type f -delete

# 必要なディレクトリを作成/確認
echo "Creating SSL directories..."
mkdir -p ssl master/ssl slave1/ssl slave2/ssl slave3/ssl

# 証明書を直接生成
echo "Generating new certificates..."
cd ssl

# CA証明書を作成
echo "Creating CA certificate..."
openssl genrsa 2048 > ca-key.pem
openssl req -new -x509 -nodes -days 3600 \
    -key ca-key.pem -out ca-cert.pem \
    -subj "/C=JP/ST=Tokyo/L=Tokyo/O=MySQL/OU=Replication/CN=MySQL_CA"

# サーバー証明書を作成
echo "Creating server certificate..."
openssl req -newkey rsa:2048 -days 3600 -nodes \
    -keyout server-key.pem -out server-req.pem \
    -subj "/C=JP/ST=Tokyo/L=Tokyo/O=MySQL/OU=Server/CN=mysql-master"
openssl rsa -in server-key.pem -out server-key.pem
openssl x509 -req -in server-req.pem -days 3600 \
    -CA ca-cert.pem -CAkey ca-key.pem -set_serial 01 -out server-cert.pem

# クライアント証明書を作成
echo "Creating client certificate..."
openssl req -newkey rsa:2048 -days 3600 -nodes \
    -keyout client-key.pem -out client-req.pem \
    -subj "/C=JP/ST=Tokyo/L=Tokyo/O=MySQL/OU=Client/CN=repl"
openssl rsa -in client-key.pem -out client-key.pem
openssl x509 -req -in client-req.pem -days 3600 \
    -CA ca-cert.pem -CAkey ca-key.pem -set_serial 02 -out client-cert.pem

# 証明書を検証
echo "Verifying certificates..."
openssl verify -CAfile ca-cert.pem server-cert.pem client-cert.pem

# アクセス権の修正
chmod 644 *.pem

cd ..

# 証明書をコピー
echo "Copying certificates to correct locations..."
cp ssl/ca-cert.pem ssl/server-cert.pem ssl/server-key.pem master/ssl/
cp ssl/ca-cert.pem ssl/client-cert.pem ssl/client-key.pem slave1/ssl/
cp ssl/ca-cert.pem ssl/client-cert.pem ssl/client-key.pem slave2/ssl/
cp ssl/ca-cert.pem ssl/client-cert.pem ssl/client-key.pem slave3/ssl/

# 証明書が正しくコピーされたか確認
echo "Verifying certificate distribution..."
ls -la master/ssl/
ls -la slave1/ssl/

echo "SSL証明書が正常に生成・配置されました。"
echo "コンテナを再起動してください: make restart"
