/* =====================================================================
   WMS_SENTINEL_DEMO  —  Synthetic warehouse-management demo database
   Generic English schema derived from a Spanish ERP data dictionary.
   100% fake data. Safe for public hackathon use.
   Target: ~2,000,000 rows in invoice_lines (fact table).
   ===================================================================== */

IF DB_ID('WMS_SENTINEL_DEMO') IS NOT NULL
BEGIN
    ALTER DATABASE WMS_SENTINEL_DEMO SET SINGLE_USER WITH ROLLBACK IMMEDIATE;
    DROP DATABASE WMS_SENTINEL_DEMO;
END
GO
CREATE DATABASE WMS_SENTINEL_DEMO;
GO
ALTER DATABASE WMS_SENTINEL_DEMO SET RECOVERY SIMPLE;
GO
USE WMS_SENTINEL_DEMO;
GO

/* ---------- Schema (generic English names) ---------- */
CREATE TABLE categories (
    category_code varchar(10)  NOT NULL PRIMARY KEY,
    description   varchar(50)  NULL,
    type          varchar(1)   NULL
);
CREATE TABLE unit_codes (
    code             varchar(5)     NOT NULL PRIMARY KEY,
    description      varchar(20)    NULL,
    conversion_value numeric(18,4)  NOT NULL DEFAULT(1)
);
CREATE TABLE warehouses (
    warehouse_id int          NOT NULL PRIMARY KEY,
    name         varchar(50)  NOT NULL,
    city         varchar(50)  NULL
);
CREATE TABLE products (
    product_code varchar(20)   NOT NULL PRIMARY KEY,
    description  varchar(200)  NULL,
    brand_id     int           NULL,
    unit         varchar(5)    NULL,
    category     varchar(10)   NULL,
    min_stock    numeric(18,2) NULL,
    sale_price   numeric(18,2) NULL,
    ean_barcode  varchar(13)   NULL,
    created_date datetime2     NULL
);
CREATE TABLE customers (
    customer_code varchar(20)   NOT NULL PRIMARY KEY,
    name          varchar(120)  NOT NULL,
    address       varchar(200)  NULL,
    city          varchar(50)   NULL,
    phone         varchar(50)   NULL,
    email         varchar(120)  NULL,
    salesperson   varchar(20)   NULL,
    credit_limit  numeric(18,2) NULL,
    credit_days   int           NULL,
    created_date  datetime2     NULL
);
CREATE TABLE stock_by_warehouse (
    product_code varchar(20)   NOT NULL,
    warehouse_id int           NOT NULL,
    on_hand_qty  numeric(18,2) NOT NULL DEFAULT(0),
    CONSTRAINT PK_stock PRIMARY KEY (product_code, warehouse_id)
);
CREATE TABLE invoices (
    invoice_id    bigint        NOT NULL PRIMARY KEY,
    doc_date      date          NOT NULL,
    doc_type      varchar(8)    NOT NULL,
    doc_number    int           NOT NULL,
    customer_code varchar(20)   NOT NULL,
    warehouse_id  int           NOT NULL,
    total         numeric(18,2) NOT NULL DEFAULT(0),
    vat_amount    numeric(18,2) NOT NULL DEFAULT(0),
    status        varchar(20)   NULL
);
CREATE TABLE invoice_lines (
    line_id      bigint        NOT NULL PRIMARY KEY,
    invoice_id   bigint        NOT NULL,
    line_no      int           NOT NULL,
    product_code varchar(20)   NOT NULL,
    quantity     numeric(18,2) NOT NULL,
    unit_value   numeric(18,2) NOT NULL,
    cost         numeric(18,2) NOT NULL,
    warehouse_id int           NOT NULL
);
GO

/* ---------- Dimensions ---------- */
-- Categories (20)
;WITH N AS (SELECT TOP (20) ROW_NUMBER() OVER (ORDER BY (SELECT NULL)) AS n
            FROM sys.all_columns)
INSERT categories (category_code, description, type)
SELECT 'CAT'+RIGHT('000'+CAST(n AS varchar),3), 'Category '+CAST(n AS varchar),
       CASE WHEN n%2=0 THEN 'A' ELSE 'B' END
FROM N;

-- Unit codes (8)
INSERT unit_codes (code, description, conversion_value) VALUES
 ('UND','Unit',1),('BOX','Box',12),('PAL','Pallet',144),('KG','Kilogram',1),
 ('LT','Liter',1),('DOZ','Dozen',12),('PCK','Pack',6),('SET','Set',1);

-- Warehouses (8)
INSERT warehouses (warehouse_id, name, city) VALUES
 (1,'Main DC','Metro City'),(2,'North Hub','Northtown'),(3,'South Hub','Southport'),
 (4,'East Depot','Eastville'),(5,'West Depot','Westburg'),(6,'Central Store','Midpoint'),
 (7,'Cold Storage','Frostville'),(8,'Returns Center','Backton');

-- Products (5,000)
;WITH N AS (SELECT TOP (5000) ROW_NUMBER() OVER (ORDER BY (SELECT NULL)) AS n
            FROM sys.all_columns a CROSS JOIN sys.all_columns b)
INSERT products (product_code, description, brand_id, unit, category, min_stock, sale_price, ean_barcode, created_date)
SELECT 'P'+RIGHT('000000'+CAST(n AS varchar),6),
       'Product '+CAST(n AS varchar),
       1+ABS(CHECKSUM(NEWID()))%50,
       'UND',
       'CAT'+RIGHT('000'+CAST(1+ABS(CHECKSUM(NEWID()))%20 AS varchar),3),
       ABS(CHECKSUM(NEWID()))%50,
       CAST(5 + ABS(CHECKSUM(NEWID()))%995 AS numeric(18,2)),
       RIGHT('0000000000000'+CAST(ABS(CHECKSUM(NEWID())) AS varchar),13),
       DATEADD(day, -(ABS(CHECKSUM(NEWID()))%1000), CAST('2026-06-06' AS date))
FROM N;

-- Customers (50,000)
;WITH N AS (SELECT TOP (50000) ROW_NUMBER() OVER (ORDER BY (SELECT NULL)) AS n
            FROM sys.all_columns a CROSS JOIN sys.all_columns b)
INSERT customers (customer_code, name, address, city, phone, email, salesperson, credit_limit, credit_days, created_date)
SELECT 'C'+RIGHT('000000'+CAST(n AS varchar),6),
       'Customer '+CAST(n AS varchar),
       CAST(n AS varchar)+' Demo Street',
       'City '+CAST(1+ABS(CHECKSUM(NEWID()))%40 AS varchar),
       '555-'+RIGHT('0000000'+CAST(ABS(CHECKSUM(NEWID()))%9999999 AS varchar),7),
       'customer'+CAST(n AS varchar)+'@example.com',
       'S'+RIGHT('000'+CAST(1+ABS(CHECKSUM(NEWID()))%50 AS varchar),3),
       CAST(ABS(CHECKSUM(NEWID()))%50000 AS numeric(18,2)),
       (ABS(CHECKSUM(NEWID()))%4)*15,
       DATEADD(day, -(ABS(CHECKSUM(NEWID()))%1500), CAST('2026-06-06' AS date))
FROM N;

-- Stock by warehouse (5,000 products x 8 warehouses = 40,000)
INSERT stock_by_warehouse (product_code, warehouse_id, on_hand_qty)
SELECT p.product_code, w.warehouse_id, CAST(ABS(CHECKSUM(NEWID()))%2000 AS numeric(18,2))
FROM products p CROSS JOIN warehouses w;

-- Invoices (200,000)
;WITH N AS (SELECT TOP (200000) ROW_NUMBER() OVER (ORDER BY (SELECT NULL)) AS n
            FROM sys.all_columns a CROSS JOIN sys.all_columns b)
INSERT invoices (invoice_id, doc_date, doc_type, doc_number, customer_code, warehouse_id, total, vat_amount, status)
SELECT n,
       DATEADD(day, -(ABS(CHECKSUM(NEWID()))%730), CAST('2026-06-06' AS date)),
       'INV',
       CAST(n AS int),
       'C'+RIGHT('000000'+CAST(1+ABS(CHECKSUM(NEWID()))%50000 AS varchar),6),
       1+ABS(CHECKSUM(NEWID()))%8,
       0, 0,
       CASE ABS(CHECKSUM(NEWID()))%4 WHEN 0 THEN 'PENDING' WHEN 1 THEN 'DISPATCHED'
            WHEN 2 THEN 'DELIVERED' ELSE 'PAID' END
FROM N;
GO
