-- Hong Kong general holidays (GovHK official lists for 2026 / 2027).
-- Idempotent: ON CONFLICT upserts on (region, holiday_date).
-- Usage: psql -d lapa -f scripts/db/04_seed_holidays_hk.sql
-- Sources:
--   https://www.gov.hk/en/about/abouthk/holiday/2026.htm
--   https://www.gov.hk/en/about/abouthk/holiday/2027.htm

INSERT INTO "Holiday" (id, region, holiday_date, name) VALUES
    -- 2026
    ('hk2026d0001', 'HK', DATE '2026-01-01', 'The first day of January'),
    ('hk2026d0002', 'HK', DATE '2026-02-17', 'Lunar New Year''s Day'),
    ('hk2026d0003', 'HK', DATE '2026-02-18', 'The second day of Lunar New Year'),
    ('hk2026d0004', 'HK', DATE '2026-02-19', 'The third day of Lunar New Year'),
    ('hk2026d0005', 'HK', DATE '2026-04-03', 'Good Friday'),
    ('hk2026d0006', 'HK', DATE '2026-04-04', 'The day following Good Friday'),
    ('hk2026d0007', 'HK', DATE '2026-04-06', 'The day following Ching Ming Festival'),
    ('hk2026d0008', 'HK', DATE '2026-04-07', 'The day following Easter Monday'),
    ('hk2026d0009', 'HK', DATE '2026-05-01', 'Labour Day'),
    ('hk2026d0010', 'HK', DATE '2026-05-25', 'The day following the Birthday of the Buddha'),
    ('hk2026d0011', 'HK', DATE '2026-06-19', 'Tuen Ng Festival'),
    ('hk2026d0012', 'HK', DATE '2026-07-01', 'Hong Kong Special Administrative Region Establishment Day'),
    ('hk2026d0013', 'HK', DATE '2026-09-26', 'The day following the Chinese Mid-Autumn Festival'),
    ('hk2026d0014', 'HK', DATE '2026-10-01', 'National Day'),
    ('hk2026d0015', 'HK', DATE '2026-10-19', 'The day following Chung Yeung Festival'),
    ('hk2026d0016', 'HK', DATE '2026-12-25', 'Christmas Day'),
    ('hk2026d0017', 'HK', DATE '2026-12-26', 'The first weekday after Christmas Day'),
    -- 2027 (second day of LNY falls on Sunday → fourth day substituted)
    ('hk2027d0001', 'HK', DATE '2027-01-01', 'The first day of January'),
    ('hk2027d0002', 'HK', DATE '2027-02-06', 'Lunar New Year''s Day'),
    ('hk2027d0003', 'HK', DATE '2027-02-08', 'The third day of Lunar New Year'),
    ('hk2027d0004', 'HK', DATE '2027-02-09', 'The fourth day of Lunar New Year'),
    ('hk2027d0005', 'HK', DATE '2027-03-26', 'Good Friday'),
    ('hk2027d0006', 'HK', DATE '2027-03-27', 'The day following Good Friday'),
    ('hk2027d0007', 'HK', DATE '2027-03-29', 'Easter Monday'),
    ('hk2027d0008', 'HK', DATE '2027-04-05', 'Ching Ming Festival'),
    ('hk2027d0009', 'HK', DATE '2027-05-01', 'Labour Day'),
    ('hk2027d0010', 'HK', DATE '2027-05-13', 'The Birthday of the Buddha'),
    ('hk2027d0011', 'HK', DATE '2027-06-09', 'Tuen Ng Festival'),
    ('hk2027d0012', 'HK', DATE '2027-07-01', 'Hong Kong Special Administrative Region Establishment Day'),
    ('hk2027d0013', 'HK', DATE '2027-09-16', 'The day following the Chinese Mid-Autumn Festival'),
    ('hk2027d0014', 'HK', DATE '2027-10-01', 'National Day'),
    ('hk2027d0015', 'HK', DATE '2027-10-08', 'Chung Yeung Festival'),
    ('hk2027d0016', 'HK', DATE '2027-12-25', 'Christmas Day'),
    ('hk2027d0017', 'HK', DATE '2027-12-27', 'The first weekday after Christmas Day')
ON CONFLICT (region, holiday_date) DO UPDATE SET name = EXCLUDED.name;
