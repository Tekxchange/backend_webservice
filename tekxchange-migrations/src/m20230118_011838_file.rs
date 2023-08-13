use crate::m20220101_000001_create_table::User;
#[cfg(not(feature = "sqlite"))]
use crate::m20230108_020136_pictures::ProductPicture;
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
                    .table(File::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(File::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(File::CreatedBy).big_integer().not_null())
                    .foreign_key(
                        ForeignKey::create()
                            .from(File::Table, File::CreatedBy)
                            .to(User::Table, User::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .col(
                        ColumnDef::new(File::CreatedAt)
                            .timestamp()
                            .not_null()
                            .extra(String::from("DEFAULT CURRENT_TIMESTAMP")),
                    )
                    .col(
                        ColumnDef::new(File::UpdatedAt)
                            .timestamp()
                            .not_null()
                            .extra(String::from("DEFAULT CURRENT_TIMESTAMP")),
                    )
                    .col(
                        ColumnDef::new(File::FileLocation)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .to_owned(),
            )
            .await?;

        #[cfg(not(feature = "sqlite"))]
        let stmt = Statement::from_string(
            manager.get_database_backend(),
            String::from(
                r#"
                    CREATE TRIGGER "file_timestamp" BEFORE INSERT OR UPDATE ON "file"
                    FOR EACH ROW EXECUTE PROCEDURE update_timestamp();
                    "#,
            ),
        );
        #[cfg(not(feature = "sqlite"))]
        manager.get_connection().execute(stmt).await?;

        #[cfg(not(feature = "sqlite"))]
        manager
            .alter_table(
                Table::alter()
                    .table(ProductPicture::Table)
                    .add_foreign_key(
                        TableForeignKey::new()
                            .from_tbl(ProductPicture::Table)
                            .to_tbl(File::Table)
                            .from_col(ProductPicture::FileId)
                            .to_col(File::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade)
                            .name("file_fk"),
                    )
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        #[cfg(not(feature = "sqlite"))]
        manager
            .alter_table(
                Table::alter()
                    .table(File::Table)
                    .drop_foreign_key(Alias::new("file_fk"))
                    .to_owned(),
            )
            .await?;

        manager
            .drop_table(Table::drop().if_exists().table(File::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum File {
    Table,
    Id,
    CreatedBy,
    CreatedAt,
    UpdatedAt,
    FileLocation,
}
