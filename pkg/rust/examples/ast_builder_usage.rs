#[cfg(feature = "sled-storage")]
mod ast_builder_usage {
    use {
        gluesql::{prelude::Glue, sled_storage::SledStorage},
        gluesql_core::ast_builder::{col, table, text, Execute},
        std::fs,
    };

    macro_rules! print_or_panic {
        ($result:expr) => {{
            match $result {
                Ok(p) => println!("{p:?}"),
                Err(e) => panic!("{e:?}"),
            };
        }};
    }
    
    pub async fn run() {
        /*
            Initiate a connection
        */
        /*
            Open a Sled database, this will create one if one does not yet exist
        */
        let sled_dir = "/tmp/gluesql/hello_world";
        fs::remove_dir_all(sled_dir).unwrap_or(());
        let storage = SledStorage::new(sled_dir).expect("Something went wrong!");
        /*
            Wrap the Sled database with Glue
        */
        let mut glue = Glue::new(storage);

        let greet_table = || table("greet");

        /*
            Create a table
        */
        let result = greet_table()
            .create_table()
            .add_column("name TEXT")
            .execute(&mut glue)
            .await;
        print_or_panic!(result);

        /*
            Insert a row
        */
        let result = greet_table()
            .insert()
            .values(vec!["'World!'"])
            .execute(&mut glue)
            .await;
        print_or_panic!(result);

        /*
            Insert multiple rows
        */
        let result = greet_table()
            .insert()
            .values(vec![
                vec![text("Glue!")],
                vec![text("AST Builder!")],
                vec![text("You!")],
            ])
            .execute(&mut glue)
            .await;
        print_or_panic!(result);

        /*
            Update a row
        */
        let result = greet_table()
            .update()
            .filter(col("name").eq(text("Glue!")))
            .set("name", text("Glue World!"))
            .execute(&mut glue)
            .await;
        print_or_panic!(result);

        /*
            Delete a row
        */
        let result = greet_table()
            .delete()
            .filter(col("name").eq(text("AST Builder!")))
            .execute(&mut glue)
            .await;
        print_or_panic!(result);

        /*
            Select all rows
        */
        let result = greet_table().select().execute(&mut glue).await;
        print_or_panic!(result);

        /*
            Select all rows with condition
        */
        let result = greet_table()
            .select()
            .filter(col("name").ilike(text("%world%")))
            .execute(&mut glue)
            .await;
        print_or_panic!(result);
    }
}

fn main() {
    #[cfg(feature = "sled-storage")]
    futures::executor::block_on(ast_builder_usage::run());
}
