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

echo "Checking master status..."
docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD -e "SHOW MASTER STATUS\G" 2>/dev/null || echo "Could not check master status"

echo "Checking slave1 status..."
docker exec mysql-slave1 mysql -u root -p$MYSQL_ROOT_PASSWORD -e "SHOW SLAVE STATUS\G" 2>/dev/null || echo "Could not check slave1 status"

echo "Resetting slave1..."
docker exec mysql-slave1 mysql -u root -p$MYSQL_ROOT_PASSWORD -e "STOP SLAVE; RESET SLAVE;" 2>/dev/null || echo "Could not reset slave1"

echo "Getting current master log position..."
MS_STATUS=$(docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD -e "SHOW MASTER STATUS" | grep mysql-bin)
CURRENT_LOG=$(echo $MS_STATUS | awk '{print $1}')
CURRENT_POS=$(echo $MS_STATUS | awk '{print $2}')

if [ -z "$CURRENT_LOG" ] || [ -z "$CURRENT_POS" ]; then
  echo "Error: Could not get master log position"
  echo "Trying to reset master..."
  docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD -e "RESET MASTER;" 2>/dev/null
  sleep 2
  MS_STATUS=$(docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD -e "SHOW MASTER STATUS" | grep mysql-bin)
  CURRENT_LOG=$(echo $MS_STATUS | awk '{print $1}')
  CURRENT_POS=$(echo $MS_STATUS | awk '{print $2}')
  
  if [ -z "$CURRENT_LOG" ] || [ -z "$CURRENT_POS" ]; then
    echo "Error: Still could not get master log position. Manual intervention required."
    exit 1
  fi
fi

echo "Current log: $CURRENT_LOG"
echo "Current position: $CURRENT_POS"

echo "Creating replication user on master if not exists..."
docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD -e "DROP USER IF EXISTS 'repl'@'%';" 2>/dev/null
docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD -e "CREATE USER 'repl'@'%' IDENTIFIED BY 'repl_password' REQUIRE SSL;" 2>/dev/null
docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD -e "GRANT REPLICATION SLAVE ON *.* TO 'repl'@'%';" 2>/dev/null
docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD -e "FLUSH PRIVILEGES;" 2>/dev/null

# Setup all slaves
for SLAVE in slave1 slave2 slave3; do
  echo "Resetting $SLAVE..."
  docker exec mysql-$SLAVE mysql -u root -p$MYSQL_ROOT_PASSWORD -e "STOP SLAVE; RESET SLAVE;" 2>/dev/null || echo "Could not reset $SLAVE"
  
  echo "Reconfiguring $SLAVE..."
  docker exec mysql-$SLAVE mysql -u root -p$MYSQL_ROOT_PASSWORD -e "CHANGE MASTER TO MASTER_HOST='mysql-master', MASTER_USER='repl', MASTER_PASSWORD='repl_password', MASTER_LOG_FILE='$CURRENT_LOG', MASTER_LOG_POS=$CURRENT_POS, MASTER_SSL=1, MASTER_SSL_CA='/etc/mysql/ssl/ca-cert.pem', MASTER_SSL_CERT='/etc/mysql/ssl/client-cert.pem', MASTER_SSL_KEY='/etc/mysql/ssl/client-key.pem', MASTER_SSL_VERIFY_SERVER_CERT=1;" 2>/dev/null || echo "Could not reconfigure $SLAVE"
  
  echo "Starting $SLAVE..."
  docker exec mysql-$SLAVE mysql -u root -p$MYSQL_ROOT_PASSWORD -e "START SLAVE;" 2>/dev/null || echo "Could not start $SLAVE"
  
  echo "Checking $SLAVE status..."
  docker exec mysql-$SLAVE mysql -u root -p$MYSQL_ROOT_PASSWORD -e "SHOW SLAVE STATUS\G" 2>/dev/null | grep -E "Slave_IO_Running:|Slave_SQL_Running:|Last_IO_Error:|Last_SQL_Error:" || echo "Could not check $SLAVE status"
done

echo "Creating test database if not exists..."
docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD -e "CREATE DATABASE IF NOT EXISTS mydb;" 2>/dev/null

echo "Recreating test table on master..."
docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD mydb -e "DROP TABLE IF EXISTS test_replication;" 2>/dev/null
docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD mydb -e "CREATE TABLE test_replication (id INT AUTO_INCREMENT PRIMARY KEY, value VARCHAR(255));" 2>/dev/null
docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD mydb -e "INSERT INTO test_replication (value) VALUES ('Test Value 1');" 2>/dev/null
docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD mydb -e "INSERT INTO test_replication (value) VALUES ('Test Value 2');" 2>/dev/null

echo "Waiting for replication to sync..."
sleep 2

echo "Checking data on master:"
docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD mydb -e "SELECT * FROM test_replication;" 2>/dev/null || echo "Could not check data on master"

for SLAVE in slave1 slave2 slave3; do
  echo "Checking data on $SLAVE:"
  docker exec mysql-$SLAVE mysql -u root -p$MYSQL_ROOT_PASSWORD mydb -e "SELECT * FROM test_replication;" 2>/dev/null || echo "Could not check data on $SLAVE"
done

echo "Fix completed."
