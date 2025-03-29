#!/bin/bash
set -e

echo "==== MySQL Replication環境の完全リセット ===="
echo "このスクリプトは全てのコンテナを停止し、データと証明書を削除して再設定します。"
echo "続行しますか？ (y/n)"
read -r response

if [[ ! "$response" =~ ^([yY][eE][sS]|[yY])$ ]]; then
    echo "操作をキャンセルしました。"
    exit 0
fi

echo "コンテナを停止しています..."
docker compose down

echo "証明書とデータを削除しています..."
rm -rf ssl/*.pem master/ssl/*.pem slave1/ssl/*.pem slave2/ssl/*.pem slave3/ssl/*.pem
rm -f ca-key.pem server-key.pem client-key.pem
rm -rf master/data/* slave1/data/* slave2/data/* slave3/data/*

echo "SSL証明書を生成しています..."
mkdir -p ssl master/ssl slave1/ssl slave2/ssl slave3/ssl
chmod +x ./.scripts/setup-ssl.sh
./.scripts/setup-ssl.sh

echo "コンテナを起動しています..."
docker compose up -d

echo "30秒待機してMySQLの起動を確認します..."
sleep 30

echo "レプリケーションを設定しています..."
chmod +x ./.scripts/fix-replication.sh
./.scripts/fix-replication.sh

echo "環境のリセットが完了しました。"
echo "レプリケーションのテストを実行するには: make test"
