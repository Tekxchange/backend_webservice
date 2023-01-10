use sea_orm_migration::{
    prelude::*,
    sea_orm::{ActiveModelTrait, ConnectionTrait, Statement},
};
use crate::m20230107_225831_products::Product;

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

        let stmt = Statement::from_string(
            manager.get_database_backend(),
            String::from(
                r#"
                            CREATE TRIGGER "category_timestamp" BEFORE INSERT OR UPDATE ON "category"
                            FOR EACH ROW EXECUTE PROCEDURE update_timestamp();
                        "#,
            ),
        );
        manager.get_connection().execute(stmt).await?;

        let seeded_category = entity::category::ActiveModel {
            category_name: sea_orm::ActiveValue::Set(String::from("Unknown")),
            ..Default::default()
        }
        .insert(manager.get_connection())
        .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Product::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Product::CategoryId)
                            .big_integer()
                            .not_null()
                            .default(seeded_category.id),
                    )
                    .add_foreign_key(
                        &TableForeignKey::new()
                            .name("fk-products-column_id")
                            .from_tbl(Product::Table)
                            .to_tbl(Category::Table)
                            .from_col(Product::CategoryId)
                            .to_col(Category::Id),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .alter_table(
                Table::alter()
                    .table(Product::Table)
                    .drop_foreign_key(Alias::new("fk-products-column_id"))
                    .drop_column(Product::CategoryId)
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
