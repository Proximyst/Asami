use ketos::io::{IoError, IoMode, SharedWrite};
use std::{
    cell::UnsafeCell,
    fmt::Arguments,
    io::{prelude::*, Cursor},
    path::Path,
};

pub struct KetosWriteWrapper<W>
where
    W: Write,
{
    write: UnsafeCell<W>,
}

impl<W> KetosWriteWrapper<W>
where
    W: Write,
{
    pub fn new(write: W) -> Self {
        KetosWriteWrapper {
            write: UnsafeCell::new(write),
        }
    }

    pub fn inner(&self) -> &W {
        unsafe { &*self.write.get() }
    }
}

impl KetosWriteWrapper<Cursor<Vec<u8>>> {
    pub fn bytearray() -> Self {
        Self::new(Cursor::new(Vec::new()))
    }

    pub fn as_string(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.inner().clone().into_inner())
    }
}

impl<W> SharedWrite for KetosWriteWrapper<W>
where
    W: Write,
{
    fn write_all(&self, buf: &[u8]) -> Result<(), IoError> {
        let mutable = unsafe { &mut *self.write.get() };
        mutable
            .write_all(buf)
            .map_err(|e| IoError::new(IoMode::Write, Path::new("<KetosWriteWrapper>"), e))?;
        Ok(())
    }

    fn write_fmt(&self, fmt: Arguments) -> Result<(), IoError> {
        let mutable = unsafe { &mut *self.write.get() };
        mutable
            .write_fmt(fmt)
            .map_err(|e| IoError::new(IoMode::Write, Path::new("<KetosWriteWrapper>"), e))?;
        Ok(())
    }

    fn flush(&self) -> Result<(), IoError> {
        let mutable = unsafe { &mut *self.write.get() };
        mutable
            .flush()
            .map_err(|e| IoError::new(IoMode::Write, Path::new("<KetosWriteWrapper>"), e))?;
        Ok(())
    }
}
