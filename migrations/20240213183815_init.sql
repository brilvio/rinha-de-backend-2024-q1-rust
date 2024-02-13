
CREATE TABLE IF NOT EXISTS users (
  id SERIAL PRIMARY KEY,
  nome VARCHAR(255) NOT NULL,
  limite BIGINT NOT NULL,
  saldo BIGINT NOT NULL,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS id_index ON users (id);

CREATE TABLE IF NOT EXISTS transactions (
  id SERIAL PRIMARY KEY,
  user_id INT NOT NULL,
  valor INT8 NOT NULL,
  tipo CHAR(1) NOT NULL,
  descricao VARCHAR(255) NOT NULL,
  realizada_em TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (user_id) REFERENCES users (id)
);

CREATE INDEX IF NOT EXISTS user_id_index ON transactions (user_id);

DO $$
DECLARE
   exist int;
BEGIN  
    select count(*) into exist from users;

    if exist = 0 then
        INSERT INTO users (nome, limite, saldo) VALUES ('user1', 100000, 0);
        INSERT INTO users (nome, limite, saldo) VALUES ('user2', 80000, 0);
        INSERT INTO users (nome, limite, saldo) VALUES ('user3', 1000000, 0);
        INSERT INTO users (nome, limite, saldo) VALUES ('user4', 10000000, 0);
        INSERT INTO users (nome, limite, saldo) VALUES ('user5', 500000, 0);
    end if;
END $$;