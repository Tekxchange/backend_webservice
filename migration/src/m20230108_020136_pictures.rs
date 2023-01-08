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
                    .table(Picture::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(Picture::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(Picture::CreatedAt)
                            .timestamp()
                            .not_null()
                            .extra(String::from("DEFAULT CURRENT_TIMESTAMP")),
                    )
                    .col(
                        ColumnDef::new(Picture::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .extra(String::from("DEFAULT CURRENT_TIMESTAMP")),
                    )
                    .col(ColumnDef::new(Picture::FileLocation).string().not_null())
                    .to_owned(),
            )
            .await?;

        let stmt = Statement::from_string(
            manager.get_database_backend(),
            String::from(
                r#"
                        CREATE TRIGGER "picture_timestamp" BEFORE INSERT OR UPDATE ON "picture"
                        FOR EACH ROW EXECUTE PROCEDURE update_timestamp();
                    "#,
            ),
        );
        manager.get_connection().execute(stmt).await?;

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
                        ColumnDef::new(ProductPicture::PictureId)
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
                            .from(ProductPicture::Table, ProductPicture::PictureId)
                            .to(Picture::Table, Picture::Id)
                            .on_delete(ForeignKeyAction::Restrict),
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
                    .index(
                        Index::create()
                            .col(ProductPicture::PictureId)
                            .col(ProductPicture::ProductId)
                            .unique(),
                    )
                    .to_owned(),
            )
            .await?;

        let stmt = Statement::from_string(
            manager.get_database_backend(),
            String::from(
                r#"
                    CREATE TRIGGER "product_picture_timestamp" BEFORE INSERT OR UPDATE ON "product_picture"
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
                    .table(ProductPicture::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().if_exists().table(Picture::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
pub enum Picture {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    FileLocation,
}

#[derive(Iden)]
pub enum ProductPicture {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    PictureId,
    ProductId,
}
