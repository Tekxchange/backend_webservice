use sea_orm_migration::{
    prelude::*,
    sea_orm::{ConnectionTrait, Statement},
};
use sea_query::table::ColumnDef;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let sql_vec = vec![
            r#"
        CREATE TABLE "user" (
            "id" BIGSERIAL PRIMARY KEY,
            "created_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            "updated_at" TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
            "username" VARCHAR(32) NOT NULL,
            "email" VARCHAR(512) NOT NULL,
            "password" VARCHAR(512) NOT NULL
        );"#,
            r#"CREATE UNIQUE INDEX idx_user_username ON "user" (username);"#,
            r#"CREATE UNIQUE INDEX idx_user_email ON "user" (email);"#,
            r#"CREATE FUNCTION update_updated_at()
        RETURNS TRIGGER AS $$
        BEGIN
            NEW.updated_at = now();
            RETURN NEW;
        END;
        $$ language 'plpgsql';"#,
            r#"CREATE TRIGGER update_user_updated_at
            BEFORE UPDATE
            ON
                "user"
            FOR EACH ROW
        EXECUTE PROCEDURE update_updated_at();"#,
        ];

        for sql in sql_vec {
            let stmt = Statement::from_string(manager.get_database_backend(), sql.to_owned());
            manager.get_connection().execute(stmt).await.map(|_| ())?
        }
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let sql_vec = vec![
            r#"DROP TRIGGER IF EXISTS "update_user_updated_at" ON "user";"#,
            r#"DROP FUNCTION IF EXISTS "update_updated_at";"#,
            r#"DROP INDEX IF EXISTS "idx_user_username";"#,
            r#"DROP INDEX IF EXISTS "idx_user_email";"#,
            r#"DROP TABLE IF EXISTS "user";"#,
        ];

        for sql in sql_vec {
            let stmt = Statement::from_string(manager.get_database_backend(), sql.to_owned());
            manager.get_connection().execute(stmt).await.map(|_| ())?
        }
        Ok(())
    }
}
