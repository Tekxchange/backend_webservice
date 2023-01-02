use sea_orm_migration::prelude::*;
use sea_query::table::ColumnDef;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                sea_query::Table::create()
                    .table(User::Table)
                    .if_not_exists()
                    .col(
                        &mut ColumnDef::new(User::Id)
                            .big_unsigned()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(&mut ColumnDef::new(User::CreatedAt).timestamp().not_null().default(sea_query::))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(sea_query::Table::drop().table(User::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum User {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    Username,
    Email,
    Password,
}
