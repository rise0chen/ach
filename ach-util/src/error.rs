use crate::MemoryState;
use core::fmt;

pub struct Error<T> {
    pub state: MemoryState,
    pub input: T,
    pub retry: bool,
}
impl<T> Error<T> {
    pub fn new(input: T) -> Self {
        Self {
            state: MemoryState::Unknown,
            input,
            retry: false,
        }
    }
}
impl<T> fmt::Debug for Error<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(&self.state, f)
    }
}

pub fn retry<I, O, F>(mut f: F, mut input: I) -> Result<O, Error<I>>
where
    F: FnMut(I) -> Result<O, Error<I>>,
{
    loop {
        match f(input) {
            Ok(val) => return Ok(val),
            Err(err) if err.retry => {
                input = err.input;
                spin_loop::spin();
                continue;
            }
            Err(err) => return Err(err),
        }
    }
}
pub fn unwrap<I, O, F>(mut f: F, mut input: I) -> O
where
    F: FnMut(I) -> Result<O, Error<I>>,
{
    loop {
        match f(input) {
            Ok(val) => return val,
            Err(err) => {
                input = err.input;
                spin_loop::spin();
                continue;
            }
        }
    }
}
