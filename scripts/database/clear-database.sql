



SET session_replication_role = replica;


DROP TABLE IF EXISTS
    _sqlx_migrations,
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
    audit_logs,
    ai_rule_applications,
    ai_memory_bank,
    ai_agent_profiles,
    ai_rules,
    social_data
CASCADE;


DROP TYPE IF EXISTS user_role CASCADE;
DROP TYPE IF EXISTS subscription_tier CASCADE;
DROP TYPE IF EXISTS social_platform CASCADE;
DROP TYPE IF EXISTS payment_status CASCADE;
DROP TYPE IF EXISTS subscription_status CASCADE;
DROP TYPE IF EXISTS invoice_status CASCADE;


SET session_replication_role = DEFAULT;


SELECT 'Database cleared successfully!' as status;
