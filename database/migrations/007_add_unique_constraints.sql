-- Migration: Add unique constraints required by ON CONFLICT clauses

-- Ensure unique triple for connected_accounts for idempotent upserts
DO $$ BEGIN
    ALTER TABLE connected_accounts
    ADD CONSTRAINT connected_accounts_user_type_identifier_unique
    UNIQUE (user_id, connector_type, account_identifier);
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

-- Ensure uniqueness for embedding_queue by document_id so ON CONFLICT works
DO $$ BEGIN
    ALTER TABLE embedding_queue
    ADD CONSTRAINT embedding_queue_document_id_unique
    UNIQUE (document_id);
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;