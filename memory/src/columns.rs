use valida_machine::{Word, MEMORY_CELL_BYTES};

pub struct MemoryCols<T: Copy> {
    pub addr: T,
    pub value: [T; MEMORY_CELL_BYTES],
}

pub trait ReadWriteLog<T: Copy, A, V> {
    fn log_read(addr: A, value: V) -> Self;
    fn log_write(addr: A, value: V) -> Self;
}

impl<T, A, V> ReadWriteLog<T, A, V> for MemoryCols<T>
where
    T: Copy,
    A: Into<T>,
    V: Into<[T; MEMORY_CELL_BYTES]>,
{
    fn log_read(addr: A, value: V) -> Self {
        Self {
            addr: addr.into(),
            value: value.into(),
        }
    }

    fn log_write(addr: A, value: V) -> Self {
        Self {
            addr: addr.into(),
            value: value.into(),
        }
    }
}
