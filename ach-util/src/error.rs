use crate::MemoryState;
pub struct Error<T> {
    pub state: MemoryState,
    pub input: T,
    pub retry: bool,
}
