#[cfg(not(feature = "sqlite"))]
use sea_orm::{ConnectionTrait, Statement};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ProductPicture::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ProductPicture::Id)
                            .big_integer()
                            .primary_key()
                            .not_null()
                            .auto_increment(),
                    )
                    .col(
                        ColumnDef::new(ProductPicture::CreatedAt)
                            .timestamp()
                            .not_null()
                            .extra(String::from("DEFAULT CURRENT_TIMESTAMP")),
                    )
                    .col(
                        ColumnDef::new(ProductPicture::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .extra(String::from("DEFAULT CURRENT_TIMESTAMP")),
                    )
                    .col(
                        ColumnDef::new(ProductPicture::FileId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ProductPicture::ProductId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ProductPicture::Table, ProductPicture::ProductId)
                            .to(
                                crate::m20230107_225831_products::Product::Table,
                                crate::m20230107_225831_products::Product::Id,
                            )
                            .on_delete(ForeignKeyAction::Restrict),
                    )
                    .to_owned(),
            )
            .await?;

        #[cfg(not(feature = "sqlite"))]
        let stmt = Statement::from_string(
            manager.get_database_backend(),
            String::from(
                r#"
                    CREATE TRIGGER "product_picture_timestamp" BEFORE INSERT OR UPDATE ON "product_picture"
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
                    .table(ProductPicture::Table)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}

#[derive(Iden)]
pub enum ProductPicture {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    FileId,
    ProductId,
}
