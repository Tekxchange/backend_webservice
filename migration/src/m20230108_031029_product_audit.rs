use crate::m20220101_000001_create_table::User;
use crate::m20230107_225831_products::Product;
use sea_orm_migration::{
    prelude::*,
    sea_orm::{ConnectionTrait, Statement},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProductAudit::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProductAudit::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
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

        let stmt = Statement::from_string(
            manager.get_database_backend(),
            String::from(
                r#"
                        CREATE TRIGGER "product_audit_timestamp" BEFORE INSERT OR UPDATE ON "product_audit"
                        FOR EACH ROW EXECUTE PROCEDURE update_timestamp();
                    "#,
            ),
        );
        manager.get_connection().execute(stmt).await?;
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
