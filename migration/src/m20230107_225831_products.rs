use super::m20220101_000001_create_table::User;
use sea_orm_migration::{
    prelude::*,
    sea_orm::{ConnectionTrait, Statement},
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts
        manager
            .create_table(
                Table::create()
                    .table(Product::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Product::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Product::ProductTitle).string().not_null())
                    .col(ColumnDef::new(Product::Description).string().not_null())
                    .col(ColumnDef::new(Product::Price).decimal().not_null())
                    .col(
                        ColumnDef::new(Product::LocationCountry)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Product::LocationState)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(Product::LocationCity)
                            .string_len(128)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Product::LocationLatitude).decimal())
                    .col(ColumnDef::new(Product::LocationLongitude).decimal())
                    .col(
                        ColumnDef::new(Product::LocationZip)
                            .string_len(64)
                            .not_null(),
                    )
                    .col(ColumnDef::new(Product::CreatedBy).big_integer().not_null())
                    .col(
                        ColumnDef::new(Product::CreatedAt)
                            .timestamp()
                            .not_null()
                            .extra(String::from("DEFAULT CURRENT_TIMESTAMP")),
                    )
                    .col(
                        ColumnDef::new(Product::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .extra(String::from("DEFAULT CURRENT_TIMESTAMP")),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-products-created_by")
                            .from(Product::Table, Product::CreatedBy)
                            .to(User::Table, User::Id),
                    )
                    .to_owned(),
            )
            .await?;

        let stmt = Statement::from_string(
            manager.get_database_backend(),
            String::from(
                r#"
                    CREATE TRIGGER "product_timestamp" BEFORE INSERT OR UPDATE ON "product"
                    FOR EACH ROW EXECUTE PROCEDURE update_timestamp();
                "#,
            ),
        );
        manager.get_connection().execute(stmt).await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Replace the sample below with your own migration scripts

        manager
            .drop_table(Table::drop().table(Product::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
#[iden = "product"]
pub enum Product {
    Table,
    Id,
    ProductTitle,
    Description,
    LocationLatitude,
    LocationLongitude,
    LocationCity,
    LocationState,
    LocationCountry,
    LocationZip,
    Price,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
}
