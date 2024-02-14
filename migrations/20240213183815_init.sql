
CREATE UNLOGGED TABLE IF NOT EXISTS users (
  id SERIAL PRIMARY KEY,
  nome VARCHAR(255) NOT NULL,
  limite BIGINT NOT NULL,
  saldo BIGINT NOT NULL,
  created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS id_index ON users (id);

CREATE UNLOGGED TABLE IF NOT EXISTS transactions (
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

-- Create trigger to update saldo
-- CREATE OR REPLACE FUNCTION update_saldo() RETURNS TRIGGER AS $$
-- DECLARE 
--     saldo_atual INT;
--     limite_atual INT;
-- BEGIN
--     select saldo, limite into saldo_atual, limite_atual from users where id = NEW.user_id;
--     IF (TG_OP = 'INSERT') THEN
--         if NEW.tipo = 'd' then
--            if (saldo_atual - NEW.valor) < (limite_atual * -1) then
--                RAISE EXCEPTION 'Saldo insuficiente';
--            end if;
--            UPDATE users SET saldo = saldo - NEW.valor WHERE id = NEW.user_id;
--         end if;
--         if NEW.tipo = 'c' then
--            UPDATE users SET saldo = saldo + NEW.valor WHERE id = NEW.user_id;    
--         end if;
--     END IF;
--     RETURN NEW;
-- END;
-- $$ LANGUAGE plpgsql;

-- CREATE TRIGGER update_saldo_insert AFTER INSERT ON transactions FOR EACH ROW EXECUTE FUNCTION update_saldo();
