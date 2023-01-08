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
                            .integer()
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
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Picture::Table).to_owned())
            .await
    }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Picture {
    Table,
    Id,
    CreatedAt,
    UpdatedAt,
    FileLocation,
}
