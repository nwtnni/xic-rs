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
        let address = self.0.len() as i64;
        if address * WORD_SIZE + size > HEAP_SIZE {
            return Err(interpret::Error::OutOfMemory)
        }
        for _ in 0..size / WORD_SIZE {
            self.0.push(rand::random());
        }
        Ok(address)
    }

    pub fn calloc(&mut self, size: i64) -> Result<i64, interpret::Error> {
        let address = self.malloc(size)?;
        for i in 0..size / WORD_SIZE {
            self.0[(address + i) as usize] = 0;
        }
        Ok(address) 
    }

    pub fn read(&self, address: i64) -> Result<i64, interpret::Error> {
        if address < 0 || address > self.0.len() as i64 {
            return Err(interpret::Error::InvalidRead(address))
        }
        Ok(self.0[address as usize])
    }

    pub fn store(&mut self, address: i64, value: i64) -> Result<(), interpret::Error> {
        if address < 0 || address > self.0.len() as i64 {
            return Err(interpret::Error::InvalidWrite(address))
        }
        self.0[address as usize] = value;
        Ok(())
    }
}
