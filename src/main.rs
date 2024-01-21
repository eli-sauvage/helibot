use dotenv::dotenv;
use errors::EnvVarError;
use sqlx::mysql::MySqlPoolOptions;
use std::env;
mod errors;
use rocket::fs::{relative, FileServer};

#[rocket::main]
async fn main() -> Result<(), errors::HelibotError> {
    let mysql_url = construct_msql_url().map_err(errors::HelibotError::EnvVarError)?;
    println!("{}", mysql_url);
    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(mysql_url.as_str())
        .await?;
    let row: (i64,) = sqlx::query_as("SELECT 150").fetch_one(&pool).await?;

    assert_eq!(row.0, 150);

    let _rocket = rocket::build()
        .mount("/", FileServer::from(relative!("views")))
        .launch()
        .await?;

    Ok(())
}

fn construct_msql_url() -> Result<String, errors::EnvVarError> {
    dotenv().unwrap();
    fn get_var(var: &str) -> Result<String, errors::EnvVarError> {
        env::var_os(var)
            .ok_or(EnvVarError::VarNotFound)?
            .into_string()
            .map_err(EnvVarError::OsString)
    }
    Ok(format!(
        "mysql://{}:{}@{}:{}/{}",
        get_var("MARIADB_USER")?,
        get_var("MARIADB_PASSWORD")?,
        get_var("MARIADB_HOST")?,
        get_var("MARIADB_PORT")?,
        get_var("MARIADB_DATABASE")?
    ))
}
