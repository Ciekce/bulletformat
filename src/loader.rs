use std::{fs::File, io::{self, BufRead, BufReader}, path::Path, marker::PhantomData};

use crate::{BulletFormat, util};

pub struct BulletFormatLoader<T: BulletFormat> {
    file: File,
    buffer_size: usize,
    marker: PhantomData<T>,
}

impl< T: BulletFormat> BulletFormatLoader<T> {
    const DATA_SIZE: usize = std::mem::size_of::<T>();

    pub fn new(path: impl AsRef<Path>, buffer_size_mb: usize) -> io::Result<Self> {
        Ok(Self {
            file: File::open(path)?,
            buffer_size: buffer_size_mb * 1024 * 1024,
            marker: PhantomData,
        })
    }

    pub fn len(&self) -> usize {
        self.file.metadata().unwrap().len() as usize / Self::DATA_SIZE
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn map_batches<F: Fn(&[T])>(self, batch_size: usize, f: F) {
        let batches_per_load = self.buffer_size / Self::DATA_SIZE / batch_size;
        let cap = Self::DATA_SIZE * batch_size * batches_per_load;

        let mut loaded = BufReader::with_capacity(cap, self.file);

        while let Ok(buf) = loaded.fill_buf() {
            if buf.is_empty() {
                break;
            }

            let data = util::to_slice_with_lifetime(buf);

            for batch in data.chunks(batch_size) {
                f(batch);
            }

            let consumed = buf.len();
            loaded.consume(consumed);
        }
    }
}