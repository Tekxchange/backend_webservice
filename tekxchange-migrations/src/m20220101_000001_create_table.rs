use crate::utils::create_trigger_on_table;
#[cfg(not(feature = "sqlite"))]
use sea_orm::{ConnectionTrait, Statement};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        #[cfg(not(feature = "sqlite"))]
        let stmt = Statement::from_string(
            manager.get_database_backend(),
            String::from(
                r#"
        CREATE OR REPLACE FUNCTION update_timestamp()
        RETURNS TRIGGER AS $$
        BEGIN
              NEW.updated_at = now(); 
              RETURN NEW;
        END;
        $$ language 'plpgsql';
        "#,
            ),
        );
        #[cfg(not(feature = "sqlite"))]
        manager.get_connection().execute(stmt).await?;

        let mut primary_key = ColumnDef::new(User::Id);

        #[cfg(not(feature = "sqlite"))]
        primary_key.big_integer();

        #[cfg(feature = "sqlite")]
        primary_key.integer();

        manager
            .create_table(
                Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(primary_key.not_null().auto_increment().primary_key())
                    .col(
                        ColumnDef::new(User::Username)
                            .string()
                            .unique_key()
                            .not_null(),
                    )
                    .col(ColumnDef::new(User::Email).string().unique_key().not_null())
                    .col(ColumnDef::new(User::Password).string().not_null())
                    .col(
                        ColumnDef::new(User::Role)
                            .small_integer()
                            .not_null()
                            .default(1 << 0),
                    )
                    .col(
                        ColumnDef::new(User::CreatedAt)
                            .timestamp()
                            .not_null()
                            .extra(String::from("DEFAULT CURRENT_TIMESTAMP")),
                    )
                    .col(
                        ColumnDef::new(User::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .extra(String::from("DEFAULT CURRENT_TIMESTAMP")),
                    )
                    .to_owned(),
            )
            .await?;

        create_trigger_on_table(User::Table, manager).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(User::Table).to_owned())
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
pub enum User {
    Table,
    Id,
    Username,
    Email,
    Role,
    Password,
    CreatedAt,
    UpdatedAt,
}
