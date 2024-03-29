//! `SeaORM` Entity. Generated by sea-orm-codegen 0.10.6

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "product")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub product_title: String,
    pub description: String,
    pub price: Decimal,
    pub location_country: String,
    pub location_state: String,
    pub location_city: String,
    pub location_latitude: Option<Decimal>,
    pub location_longitude: Option<Decimal>,
    pub location_zip: String,
    pub created_by: i64,
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::product_audit::Entity")]
    ProductAudit,
    #[sea_orm(has_many = "super::product_category::Entity")]
    ProductCategory,
    #[sea_orm(has_many = "super::product_picture::Entity")]
    ProductPicture,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::CreatedBy",
        to = "super::user::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    User,
}

impl Related<super::product_audit::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProductAudit.def()
    }
}

impl Related<super::product_category::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProductCategory.def()
    }
}

impl Related<super::product_picture::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProductPicture.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::User.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
