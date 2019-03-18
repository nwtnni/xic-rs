use crate::interpret;

#[derive(Debug)]
pub struct Heap(Vec<i64>);

const WORD_SIZE: i64 = 8;
const HEAP_SIZE: i64 = 10240;

impl Heap {
    pub fn malloc(&mut self, size: i64) -> Result<i64, interpret::Error> {
        if size < 0 || size % WORD_SIZE != 0 {
            return Err(interpret::Error::InvalidMalloc(size))
        }
        let address = self.0.len() as i64 * WORD_SIZE;
        if address + size > HEAP_SIZE {
            return Err(interpret::Error::OutOfMemory)
        }
        for _ in 0..size / WORD_SIZE {
            self.0.push(rand::random());
        }
        Ok(address)
    }

    pub fn calloc(&mut self, size: i64) -> Result<i64, interpret::Error> {
        let address = self.malloc(size)?;
        let index = address / WORD_SIZE;
        for i in 0..size / WORD_SIZE {
            self.0[(index + i) as usize] = 0;
        }
        Ok(address) 
    }

    pub fn read(&self, address: i64) -> Result<i64, interpret::Error> {
        let index = self.address_to_index(address)?;
        Ok(self.0[index])
    }

    pub fn store(&mut self, address: i64, value: i64) -> Result<(), interpret::Error> {
        let index = self.address_to_index(address)?;
        self.0[index] = value;
        Ok(())
    }

    fn address_to_index(&self, address: i64) -> Result<usize, interpret::Error> {
        let index = (address / WORD_SIZE) as usize;
        if address < 0 || address % WORD_SIZE != 0 || index > self.0.len() {
            return Err(interpret::Error::InvalidRead(address))
        } else {
            Ok(index)
        }
    }
}
