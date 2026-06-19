pub mod postgres;
pub mod mysql;

#[derive(PartialEq)]
pub enum DatabaseType {
    Postgres,
    Mysql,
}