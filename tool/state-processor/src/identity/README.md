### Process steps
1. take `Identity::IdentityOf`, `Identity::Registrars`, `Identity::SubsOf`
2. free super_id's reservation
3. adjust registrations and set `AccountMigration::Identities`
4. truncate registrar account id and adjust registrars fee decimal
5. set `Identity::Registrars
