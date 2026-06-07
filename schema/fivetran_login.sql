/* Dedicated least-privilege login for Fivetran.
   Scoped ONLY to WMS_SENTINEL_DEMO (synthetic data).
   It CANNOT see VENEPAC (real company data). */
USE master;
IF SUSER_ID('fivetran_svc') IS NOT NULL DROP LOGIN fivetran_svc;
-- Replace the placeholder below with your own strong password before running.
CREATE LOGIN fivetran_svc WITH PASSWORD = 'CHANGE_ME_StrongPassword!', CHECK_POLICY = OFF;
GO
USE WMS_SENTINEL_DEMO;
IF USER_ID('fivetran_svc') IS NOT NULL DROP USER fivetran_svc;
CREATE USER fivetran_svc FOR LOGIN fivetran_svc;
ALTER ROLE db_datareader ADD MEMBER fivetran_svc;
GRANT VIEW DEFINITION TO fivetran_svc;
GO
