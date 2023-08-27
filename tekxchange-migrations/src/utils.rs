#[allow(unused_imports)]
use sea_orm_migration::{
    sea_orm::{ConnectionTrait, Statement},
    sea_query::Iden,
    DbErr, SchemaManager,
};

#[allow(unused_variables)]
pub async fn create_trigger_on_table(
    table: impl Iden,
    manager: &SchemaManager<'_>,
) -> Result<(), DbErr> {
    #[cfg(not(feature = "sqlite"))]
    {
        let mut table_name = String::new();
        table.unquoted(&mut table_name);

        #[allow(unused_variables)]
        let stmt = Statement::from_string(
            manager.get_database_backend(),
            format!(
                r#"
                CREATE TRIGGER "{0}_timestamp" BEFORE INSERT OR UPDATE ON "{0}"
                FOR EACH ROW EXECUTE PROCEDURE update_timestamp()
                "#,
                table_name
            ),
        );
        manager.get_connection().execute(stmt).await?;
    }

    Ok(())
}
