-- 既存の repl ユーザーがいれば削除
DROP USER IF EXISTS 'repl'@'%';

-- SSL接続を要求するレプリケーションユーザーを作成
CREATE USER 'repl'@'%' IDENTIFIED BY 'repl_password' 
REQUIRE SSL;

GRANT REPLICATION SLAVE ON *.* TO 'repl'@'%';
FLUSH PRIVILEGES;
