-- Clear all ConHub database records
-- WARNING: This will delete ALL data from the database

-- Disable foreign key checks temporarily
SET session_replication_role = replica;

-- Clear all tables (in reverse dependency order)
TRUNCATE TABLE 
    social_tokens,
    social_connections,
    users,
    billing_subscriptions,
    billing_invoices,
    billing_payment_methods,
    ai_agents,
    rulesets,
    rules,
    agent_ruleset_connections,
    data_sources,
    documents,
    repositories,
    urls,
    api_tokens,
    webhooks,
    audit_logs
CASCADE;

-- Re-enable foreign key checks
SET session_replication_role = DEFAULT;

-- Reset sequences
ALTER SEQUENCE IF EXISTS users_id_seq RESTART WITH 1;
ALTER SEQUENCE IF EXISTS social_connections_id_seq RESTART WITH 1;
ALTER SEQUENCE IF EXISTS social_tokens_id_seq RESTART WITH 1;
ALTER SEQUENCE IF EXISTS billing_subscriptions_id_seq RESTART WITH 1;
ALTER SEQUENCE IF EXISTS billing_invoices_id_seq RESTART WITH 1;
ALTER SEQUENCE IF EXISTS billing_payment_methods_id_seq RESTART WITH 1;
ALTER SEQUENCE IF EXISTS ai_agents_id_seq RESTART WITH 1;
ALTER SEQUENCE IF EXISTS rulesets_id_seq RESTART WITH 1;
ALTER SEQUENCE IF EXISTS rules_id_seq RESTART WITH 1;
ALTER SEQUENCE IF EXISTS agent_ruleset_connections_id_seq RESTART WITH 1;
ALTER SEQUENCE IF EXISTS data_sources_id_seq RESTART WITH 1;
ALTER SEQUENCE IF EXISTS documents_id_seq RESTART WITH 1;
ALTER SEQUENCE IF EXISTS repositories_id_seq RESTART WITH 1;
ALTER SEQUENCE IF EXISTS urls_id_seq RESTART WITH 1;
ALTER SEQUENCE IF EXISTS api_tokens_id_seq RESTART WITH 1;
ALTER SEQUENCE IF EXISTS webhooks_id_seq RESTART WITH 1;
ALTER SEQUENCE IF EXISTS audit_logs_id_seq RESTART WITH 1;

-- Display confirmation
SELECT 'Database cleared successfully!' as status;
