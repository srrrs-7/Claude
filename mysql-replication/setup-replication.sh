#!/bin/bash
set -e

# Check if password is provided via environment variable or use default
if [ -z "${MYSQL_ROOT_PASSWORD}" ]; then
  # Try to load from .env file
  if [ -f .env ]; then
    export $(grep -v '^#' .env | xargs)
  fi
  
  # If still not set, prompt for password
  if [ -z "${MYSQL_ROOT_PASSWORD}" ]; then
    echo -n "Enter MySQL root password: "
    read -s MYSQL_ROOT_PASSWORD
    echo ""
  fi
fi

echo "Setting up SSL certificates..."
chmod +x setup-ssl.sh
./setup-ssl.sh

echo "Waiting for MySQL Master to be ready..."
until docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD -e "SELECT 1" 2>/dev/null; do
    echo "Waiting for MySQL Master to be ready..."
    sleep 1
done

echo "Waiting for MySQL Slave 1 to be ready..."
until docker exec mysql-slave1 mysql -u root -p$MYSQL_ROOT_PASSWORD -e "SELECT 1" 2>/dev/null; do
    echo "Waiting for MySQL Slave 1 to be ready..."
    sleep 1
done

echo "Waiting for MySQL Slave 2 to be ready..."
until docker exec mysql-slave2 mysql -u root -p$MYSQL_ROOT_PASSWORD -e "SELECT 1" 2>/dev/null; do
    echo "Waiting for MySQL Slave 2 to be ready..."
    sleep 1
done

echo "Waiting for MySQL Slave 3 to be ready..."
until docker exec mysql-slave3 mysql -u root -p$MYSQL_ROOT_PASSWORD -e "SELECT 1" 2>/dev/null; do
    echo "Waiting for MySQL Slave 3 to be ready..."
    sleep 1
done

echo "Getting master status..."
MS_STATUS=$(docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD -e "SHOW MASTER STATUS" 2>/dev/null | grep mysql-bin)
CURRENT_LOG=$(echo $MS_STATUS | awk '{print $1}')
CURRENT_POS=$(echo $MS_STATUS | awk '{print $2}')

echo "Current log: $CURRENT_LOG"
echo "Current position: $CURRENT_POS"

# Check SSL status on master
echo "Checking SSL status on master:"
docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD -e "SHOW VARIABLES LIKE '%ssl%';" 2>/dev/null

# Setup all slaves with SSL
for SLAVE in slave1 slave2 slave3; do
  echo "Setting up $SLAVE with SSL..."
  docker exec mysql-$SLAVE mysql -u root -p$MYSQL_ROOT_PASSWORD -e "STOP SLAVE;" 2>/dev/null
  docker exec mysql-$SLAVE mysql -u root -p$MYSQL_ROOT_PASSWORD -e "RESET SLAVE;" 2>/dev/null
  docker exec mysql-$SLAVE mysql -u root -p$MYSQL_ROOT_PASSWORD -e "CHANGE MASTER TO MASTER_HOST='mysql-master', MASTER_USER='repl', MASTER_PASSWORD='repl_password', MASTER_LOG_FILE='$CURRENT_LOG', MASTER_LOG_POS=$CURRENT_POS, MASTER_SSL=1, MASTER_SSL_CA='/etc/mysql/ssl/ca-cert.pem', MASTER_SSL_CERT='/etc/mysql/ssl/client-cert.pem', MASTER_SSL_KEY='/etc/mysql/ssl/client-key.pem';" 2>/dev/null
  docker exec mysql-$SLAVE mysql -u root -p$MYSQL_ROOT_PASSWORD -e "START SLAVE;" 2>/dev/null
  
  echo "Checking $SLAVE SSL status..."
  docker exec mysql-$SLAVE mysql -u root -p$MYSQL_ROOT_PASSWORD -e "SHOW SLAVE STATUS\G" 2>/dev/null | grep -i "ssl"
done

echo "Creating test database if not exists..."
docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD -e "CREATE DATABASE IF NOT EXISTS mydb;" 2>/dev/null

echo "Creating test table on master..."
docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD mydb -e "DROP TABLE IF EXISTS test_replication;" 2>/dev/null
docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD mydb -e "CREATE TABLE test_replication (id INT AUTO_INCREMENT PRIMARY KEY, value VARCHAR(255));" 2>/dev/null
docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD mydb -e "INSERT INTO test_replication (value) VALUES ('Test Value 1 with SSL');" 2>/dev/null
docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD mydb -e "INSERT INTO test_replication (value) VALUES ('Test Value 2 with SSL');" 2>/dev/null

echo "Waiting for replication to sync..."
sleep 10

echo "Checking data on master:"
docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD mydb -e "SELECT * FROM test_replication;" 2>/dev/null

for SLAVE in slave1 slave2 slave3; do
  echo "Checking data on $SLAVE:"
  docker exec mysql-$SLAVE mysql -u root -p$MYSQL_ROOT_PASSWORD mydb -e "SELECT * FROM test_replication;" 2>/dev/null
done

echo "SSL Replication setup completed."
