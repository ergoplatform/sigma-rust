#[derive(PartialEq, Eq, Debug)]
pub enum NumOp {
    Add,
}

#[derive(PartialEq, Eq, Debug)]
pub enum BinOp {
    Num(NumOp),
}
