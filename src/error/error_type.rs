#[derive(Debug)]
pub enum ErrorType<C, R> {
    COMPILETIME(C),
    RUNTIME(R)
}