use crate::m20230107_225831_products::Product;
#[cfg(not(feature = "sqlite"))]
use sea_orm::{ConnectionTrait, Statement};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .create_table(
                Table::create()
                    .table(Category::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Category::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Category::CreatedAt)
                            .timestamp()
                            .not_null()
                            .extra(String::from("DEFAULT CURRENT_TIMESTAMP")),
                    )
                    .col(
                        ColumnDef::new(Category::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .extra(String::from("DEFAULT CURRENT_TIMESTAMP")),
                    )
                    .col(
                        ColumnDef::new(Category::CategoryName)
                            .unique_key()
                            .not_null()
                            .string_len(128),
                    )
                    .to_owned(),
            )
            .await?;

        #[cfg(not(feature = "sqlite"))]
        let stmt = Statement::from_string(
            manager.get_database_backend(),
            String::from(
                r#"
                CREATE TRIGGER "category_timestamp" BEFORE INSERT OR UPDATE ON "category"
                FOR EACH ROW EXECUTE PROCEDURE update_timestamp();
                "#,
            ),
        );
        #[cfg(not(feature = "sqlite"))]
        manager.get_connection().execute(stmt).await?;

        manager
            .create_table(
                Table::create()
                    .table(ProductCategory::Table)
                    .col(
                        ColumnDef::new(ProductCategory::Id)
                            .big_integer()
                            .primary_key()
                            .not_null()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(ProductCategory::CreatedAt)
                            .timestamp()
                            .not_null()
                            .extra(String::from("DEFAULT CURRENT_TIMESTAMP")),
                    )
                    .col(
                        ColumnDef::new(ProductCategory::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .extra(String::from("DEFAULT CURRENT_TIMESTAMP")),
                    )
                    .col(
                        ColumnDef::new(ProductCategory::ProductId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProductCategory::CategoryId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProductCategory::PriorityIndex)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .index(
                        Index::create()
                            .name("product-category-product_priority_index")
                            .col(ProductCategory::ProductId)
                            .col(ProductCategory::PriorityIndex)
                            .unique(),
                    )
                    .index(
                        Index::create()
                            .name("product-category-product_category_index")
                            .col(ProductCategory::CategoryId)
                            .col(ProductCategory::ProductId)
                            .unique(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProductCategory::Table, ProductCategory::CategoryId)
                            .to(Category::Table, Category::Id),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProductCategory::Table, ProductCategory::ProductId)
                            .to(Product::Table, Product::Id),
                    )
                    .to_owned(),
            )
            .await?;

        #[cfg(not(feature = "sqlite"))]
        let stmt = Statement::from_string(
            manager.get_database_backend(),
            String::from(
                r#"
                CREATE TRIGGER "product_category_timestamp" BEFORE INSERT OR UPDATE ON "product_category"
                FOR EACH ROW EXECUTE PROCEDURE update_timestamp();
            "#,
            ),
        );
        #[cfg(not(feature = "sqlite"))]
        manager.get_connection().execute(stmt).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .if_exists()
                    .table(ProductCategory::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().if_exists().table(Category::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Category {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    CategoryName,
}

#[derive(Iden)]
enum ProductCategory {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    ProductId,
    CategoryId,
    PriorityIndex,
}
