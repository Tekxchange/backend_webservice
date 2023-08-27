use sea_orm_migration::prelude::*;

use crate::utils::create_trigger_on_table;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let mut primary_key = ColumnDef::new(ProductPicture::Id);

        #[cfg(not(feature = "sqlite"))]
        primary_key.big_integer();

        #[cfg(feature = "sqlite")]
        primary_key.integer();

        manager
            .create_table(
                Table::create()
                    .table(ProductPicture::Table)
                    .if_not_exists()
                    .col(primary_key.primary_key().not_null().auto_increment())
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

        create_trigger_on_table(ProductPicture::Table, manager).await?;

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
