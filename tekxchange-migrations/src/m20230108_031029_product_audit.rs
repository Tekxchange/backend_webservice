use crate::m20220101_000001_create_table::User;
use crate::m20230107_225831_products::Product;
use crate::utils::create_trigger_on_table;
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let mut primary_key = ColumnDef::new(ProductAudit::Id);

        #[cfg(not(feature = "sqlite"))]
        primary_key.big_integer();

        #[cfg(feature = "sqlite")]
        primary_key.integer();

        manager
            .create_table(
                Table::create()
                    .table(ProductAudit::Table)
                    .if_not_exists()
                    .col(primary_key.not_null().auto_increment().primary_key())
                    .col(
                        ColumnDef::new(ProductAudit::CreatedAt)
                            .timestamp()
                            .not_null()
                            .extra(String::from("DEFAULT CURRENT_TIMESTAMP")),
                    )
                    .col(
                        ColumnDef::new(ProductAudit::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .extra(String::from("DEFAULT CURRENT_TIMESTAMP")),
                    )
                    .col(
                        ColumnDef::new(ProductAudit::ReviewerId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProductAudit::ProductId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProductAudit::ReviewStatus)
                            .small_integer()
                            .not_null(),
                    )
                    .col(ColumnDef::new(ProductAudit::ReviewText).text())
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProductAudit::Table, ProductAudit::ReviewerId)
                            .to(User::Table, User::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProductAudit::Table, ProductAudit::ProductId)
                            .to(Product::Table, Product::Id),
                    )
                    .to_owned(),
            )
            .await?;

        create_trigger_on_table(ProductAudit::Table, manager).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .if_exists()
                    .table(ProductAudit::Table)
                    .to_owned(),
            )
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum ProductAudit {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    ReviewerId,
    ProductId,
    ReviewStatus,
    ReviewText,
}
