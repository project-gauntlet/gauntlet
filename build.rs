fn main() {
    println!("cargo:rerun-if-changed=src/db_migrations");
}
