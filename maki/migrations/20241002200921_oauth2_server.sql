-- Add migration script here
ALTER TABLE accounts ADD CONSTRAINT unique_provider_account_id UNIQUE ("providerAccountId");

-- Change the name of the session token column to refresh_token
ALTER TABLE sessions RENAME COLUMN "sessionToken" TO refresh_token;
