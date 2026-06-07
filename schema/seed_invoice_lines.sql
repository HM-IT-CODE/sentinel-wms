USE WMS_SENTINEL_DEMO;
SET NOCOUNT ON;
DECLARE @i bigint = 0, @batch int = 100000, @target bigint = 2000000;
WHILE @i < @target
BEGIN
    ;WITH N AS (
        SELECT TOP (@batch) ROW_NUMBER() OVER (ORDER BY (SELECT NULL)) + @i AS id
        FROM sys.all_columns a CROSS JOIN sys.all_columns b
    )
    INSERT invoice_lines (line_id, invoice_id, line_no, product_code, quantity, unit_value, cost, warehouse_id)
    SELECT id,
           ((id-1)/10)+1,
           CAST(((id-1)%10)+1 AS int),
           'P'+RIGHT('000000'+CAST(1+ABS(CHECKSUM(NEWID()))%5000 AS varchar),6),
           CAST(1+ABS(CHECKSUM(NEWID()))%50 AS numeric(18,2)),
           CAST(5+ABS(CHECKSUM(NEWID()))%995 AS numeric(18,2)),
           CAST(3+ABS(CHECKSUM(NEWID()))%700 AS numeric(18,2)),
           1+ABS(CHECKSUM(NEWID()))%8
    FROM N;
    SET @i = @i + @batch;
END
SELECT COUNT(*) AS total_invoice_lines FROM invoice_lines;
