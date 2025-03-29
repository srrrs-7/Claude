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

echo "Creating test table on master..."
docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD mydb -e "CREATE TABLE IF NOT EXISTS test_replication (id INT AUTO_INCREMENT PRIMARY KEY, value VARCHAR(255));"

echo "Inserting test data into master..."
docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD mydb -e "INSERT INTO test_replication (value) VALUES ('Test Value 1');"
docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD mydb -e "INSERT INTO test_replication (value) VALUES ('Test Value 2');"

echo "Waiting for replication to sync..."
sleep 5

echo "Checking data on master:"
docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD mydb -e "SELECT * FROM test_replication;"

echo "Checking data on slave 1:"
docker exec mysql-slave1 mysql -u root -p$MYSQL_ROOT_PASSWORD mydb -e "SELECT * FROM test_replication;"

echo "Checking data on slave 2:"
docker exec mysql-slave2 mysql -u root -p$MYSQL_ROOT_PASSWORD mydb -e "SELECT * FROM test_replication;"

echo "Checking data on slave 3:"
docker exec mysql-slave3 mysql -u root -p$MYSQL_ROOT_PASSWORD mydb -e "SELECT * FROM test_replication;"

# Check SSL status
echo "Checking SSL status on replication connections:"
for SLAVE in slave1 slave2 slave3; do
  echo "SSL status on $SLAVE:"
  docker exec mysql-$SLAVE mysql -u root -p$MYSQL_ROOT_PASSWORD -e "SHOW SLAVE STATUS\G" 2>/dev/null | grep -i "Using_Ssl"
done

echo "Replication test completed."
