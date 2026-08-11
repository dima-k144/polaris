#[derive(Debug)]
pub enum Expr {
    Symbol(String),
    Number(f64),
    FlatList(usize)
}
