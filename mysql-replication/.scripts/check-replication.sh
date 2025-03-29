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

echo "==== Master Status ===="
docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD -e "SHOW MASTER STATUS\G" 2>/dev/null || echo "Failed to check master status"
echo ""

for SLAVE in slave1 slave2 slave3; do
  echo "==== $SLAVE Status ===="
  docker exec mysql-$SLAVE mysql -u root -p$MYSQL_ROOT_PASSWORD -e "SHOW SLAVE STATUS\G" 2>/dev/null || echo "Failed to check $SLAVE status"
  echo ""
  
  echo "==== $SLAVE IO and SQL threads ===="
  docker exec mysql-$SLAVE mysql -u root -p$MYSQL_ROOT_PASSWORD -e "SHOW PROCESSLIST\G" 2>/dev/null | grep -i "replica" || echo "No replica threads found in $SLAVE"
  echo ""
  
  echo "==== $SLAVE SSL Status ===="
  docker exec mysql-$SLAVE mysql -u root -p$MYSQL_ROOT_PASSWORD -e "SHOW SLAVE STATUS\G" 2>/dev/null | grep -i -A 5 "Using_Ssl" || echo "Failed to get SSL status"
  echo ""
  
  echo "==== $SLAVE Error Log (Last 10 lines) ===="
  docker exec mysql-$SLAVE bash -c "tail -n 10 /var/log/mysql/error.log 2>/dev/null || echo 'No error log found'"
  echo ""
done

echo "==== Checking for Tables in Master ===="
docker exec mysql-master mysql -u root -p$MYSQL_ROOT_PASSWORD -e "SHOW DATABASES; USE mydb; SHOW TABLES;" 2>/dev/null || echo "Failed to check tables in master"
echo ""

for SLAVE in slave1 slave2 slave3; do
  echo "==== Checking for Tables in $SLAVE ===="
  docker exec mysql-$SLAVE mysql -u root -p$MYSQL_ROOT_PASSWORD -e "SHOW DATABASES; USE mydb; SHOW TABLES;" 2>/dev/null || echo "Failed to check tables in $SLAVE"
  echo ""
done

echo "Replication check completed."
