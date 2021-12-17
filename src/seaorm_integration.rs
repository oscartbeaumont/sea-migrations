use sea_orm::{
    sea_query::{Alias, ColumnDef, ForeignKey, ForeignKeyCreateStatement, TableRef},
    ColumnTrait, ColumnType, EntityTrait, Iden, PrimaryKeyToColumn, PrimaryKeyTrait, RelationTrait,
    RelationType,
};

// CustomColumnDef is a copy of the struct defined at https://github.com/SeaQL/sea-orm/blob/master/src/entity/column.rs#L7 with all fields set to public.
// It exists so that the unsafe transmutate operation can be applied to access private fields on the struct.
// This is a TEMPORARY solution and I will ask if these values can be directly exposed by sea_query in the future. This solution relies on internal implementation details of sea_query and unsafe code which is not good!
struct CustomColumnDef {
    pub col_type: ColumnType,
    pub null: bool,
    pub unique: bool,
    pub indexed: bool,
}

// get_column_def is used to convert between the sea_orm Column and the sea_query ColumnDef.
pub(crate) fn get_column_def<T: EntityTrait>(column: T::Column) -> ColumnDef {
    let column_def_prelude: CustomColumnDef = unsafe { std::mem::transmute(column.def()) }; // Note: This is used to access private fields and hence relies on internal implementation details of sea_query and unsafe code which is not good!
    let mut column_def =
        ColumnDef::new_with_type(column, column_def_prelude.col_type.clone().into());
    if !column_def_prelude.null {
        column_def.not_null();
    }
    if column_def_prelude.unique {
        column_def.unique_key();
    }
    if column_def_prelude.indexed {
        panic!("Indexed columns are not yet able to be migrated!");
    }

    if let Some(_) = T::PrimaryKey::from_column(column) {
        column_def.primary_key();

        if T::PrimaryKey::auto_increment() && column_def_prelude.col_type == ColumnType::Integer {
            column_def.auto_increment();
        }
    }

    column_def
}

// get_column_foreign_key_def is used to convert between the sea_orm Relation and the sea_query ForeignKey.
pub(crate) fn get_column_foreign_key_def<T: EntityTrait>(
    relation: T::Relation,
) -> ForeignKeyCreateStatement {
    let rel_def = relation.def();
    match rel_def.rel_type {
        RelationType::HasOne => {
            let mut foreign_key = ForeignKey::create()
                .from(
                    table_ref_to_alias(rel_def.from_tbl),
                    Alias::new(&rel_def.from_col.to_string()),
                )
                .to(
                    table_ref_to_alias(rel_def.to_tbl),
                    Alias::new(&rel_def.to_col.to_string()),
                )
                .to_owned();

            if let Some(fk_action) = rel_def.on_delete {
                foreign_key.on_delete(fk_action);
            }

            if let Some(fk_action) = rel_def.on_update {
                foreign_key.on_update(fk_action);
            }

            foreign_key
        }
        _ => panic!(
            "Sea migrations does not yet support '{:?}' relationships!",
            rel_def.rel_type
        ),
    }
}

// table_ref_to_alias converts between a sea-query TableRef and a sea-query Alias.
pub(crate) fn table_ref_to_alias(table_ref: TableRef) -> Alias {
    match table_ref {
        TableRef::Table(iden) => Alias::new(&iden.to_string()),
        // TableRef::SchemaTable
        // TableRef::TableAlias
        // TableRef::SchemaTableAlias
        // TableRef::SubQuery
        // TODO: Support all TableRef types.
        _ => panic!(
            "Sea migrations does not yet support '{:?}' TableRef!",
            table_ref
        ),
    }
}
