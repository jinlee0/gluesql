use crate::storage::Store;
use crate::translator::{CommandType, Filter, Row, SelectTranslation};
use nom_sql::InsertStatement;
use std::fmt::Debug;

pub enum Payload<T: Debug> {
    Create,
    Insert(Row<T>),
    Select(Box<dyn Iterator<Item = Row<T>>>),
    Delete(usize),
    Update(usize),
}

fn execute_get_data<T: 'static>(
    storage: &dyn Store<T>,
    table_name: &str,
    filter: Filter,
) -> Box<dyn Iterator<Item = Row<T>>>
where
    T: Debug,
{
    let rows = storage
        .get_data(&table_name)
        .unwrap()
        .filter(move |row| filter.check(row));

    Box::new(rows)
}

pub fn execute_select<T: 'static>(
    storage: &dyn Store<T>,
    translation: SelectTranslation,
) -> Box<dyn Iterator<Item = Row<T>>>
where
    T: Debug,
{
    let SelectTranslation {
        table_name,
        blend,
        filter,
        limit,
    } = translation;

    let rows = execute_get_data(storage, &table_name, filter)
        .enumerate()
        .filter(move |(i, _)| limit.check(i))
        .map(|(_, row)| row)
        .map(move |row| {
            let Row { key, items } = row;
            let items = items.into_iter().filter(|item| blend.check(item)).collect();

            Row { key, items }
        });

    Box::new(rows)
}

pub fn execute<T: 'static>(
    storage: &dyn Store<T>,
    command_type: CommandType,
) -> Result<Payload<T>, ()>
where
    T: Debug,
{
    let payload = match command_type {
        CommandType::Create(statement) => {
            storage.set_schema(statement).unwrap();

            Payload::Create
        }
        CommandType::Select(translation) => {
            let rows = execute_select(storage, translation);

            Payload::Select(Box::new(rows))
        }
        CommandType::Insert(insert_statement) => {
            let (table_name, insert_fields, insert_data) = match insert_statement {
                InsertStatement {
                    table,
                    fields,
                    data,
                    ..
                } => (table.name, fields, data),
            };
            let create_fields = storage.get_schema(&table_name).unwrap().fields;
            let key = storage.gen_id().unwrap();
            let row = Row::from((key, create_fields, insert_fields, insert_data));

            let row = storage.set_data(&table_name, row).unwrap();

            Payload::Insert(row)
        }
        CommandType::Delete { table_name, filter } => {
            let num_rows = execute_get_data(storage, &table_name, filter).fold(0, |num, row| {
                storage.del_data(&table_name, &row.key).unwrap();

                num + 1
            });

            Payload::Delete(num_rows)
        }
        CommandType::Update {
            table_name,
            update,
            filter,
        } => {
            let num_rows = execute_get_data(storage, &table_name, filter)
                .map(|row| update.apply(row))
                .fold(0, |num, row| {
                    storage.set_data(&table_name, row).unwrap();

                    num + 1
                });

            Payload::Update(num_rows)
        }
    };

    Ok(payload)
}