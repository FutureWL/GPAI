-- 0002_seed.sql
-- 骨架阶段最小种子:NASDAQ 交易所 + AAPL + 2 个数据源
INSERT INTO exchanges (code, name_zh, name_en, market, timezone, primary_currency) VALUES
  ('NASDAQ', '纳斯达克', 'NASDAQ', 3, 'America/New_York', 'USD'),
  ('NYSE',   '纽约证券交易所', 'New York Stock Exchange', 3, 'America/New_York', 'USD'),
  ('SH',     '上海证券交易所', 'Shanghai Stock Exchange', 1, 'Asia/Shanghai', 'CNY'),
  ('SZ',     '深圳证券交易所', 'Shenzhen Stock Exchange', 1, 'Asia/Shanghai', 'CNY'),
  ('HK',     '香港交易所', 'Hong Kong Exchange', 2, 'Asia/Hong_Kong', 'HKD');

INSERT INTO instruments (id, market, symbol, exchange_code, name_zh, name_en, asset_class, currency, timezone, lot_size) VALUES
  ('US.AAPL.NASDAQ', 3, 'AAPL', 'NASDAQ', '苹果', 'Apple Inc.', 1, 'USD', 'America/New_York', 1);

INSERT INTO data_sources (id, display_name, enabled) VALUES
  ('yahoo', 'Yahoo Finance', TRUE),
  ('mock',  'Mock Source',   TRUE);