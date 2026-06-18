#[salsa::db]
pub struct Jar(MyInput);

pub trait Db: salsa::DbWithJar<Jar> {}

#[salsa::input]
pub struct MyInput {
    pub text: String,
}

#[salsa::db]
pub struct Database {
    storage: salsa::Storage<Self>,
}

impl salsa::Database for Database {}
impl Db for Database {}

fn main() {
    let _db = Database::default();
}
