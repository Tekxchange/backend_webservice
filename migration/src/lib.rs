pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20230107_225831_products;
mod m20230108_020136_pictures;
mod m20230108_031029_product_audit;
mod m20230109_234237_category;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20220101_000001_create_table::Migration),
            Box::new(m20230107_225831_products::Migration),
            Box::new(m20230108_020136_pictures::Migration),
            Box::new(m20230108_031029_product_audit::Migration),
            Box::new(m20230109_234237_category::Migration),
        ]
    }
}
