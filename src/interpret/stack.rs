use std::collections::HashMap;

use crate::interpret;
use crate::data::operand;

#[derive(Debug)]
pub struct Stack(Vec<Frame>);

impl Stack {
    pub fn new() -> Self {
        Stack(vec![Default::default()])
    }

    pub fn push(&mut self) {
        self.0.push(Default::default());
    }

    pub fn pop(&mut self) {
        self.0.pop();
    }

    pub fn current(&self) -> &Frame {
        self.0.last().unwrap()
    }

    pub fn current_mut(&mut self) -> &mut Frame {
        self.0.last_mut().unwrap()
    }

    pub fn parent_mut(&mut self) -> &mut Frame {
        let parent = self.0.len() - 2;
        self.0.get_mut(parent).unwrap()
    }
}

#[derive(Default, Debug)]
pub struct Frame(HashMap<operand::Temp, i64>);

impl Frame {
    pub fn get(&self, temp: &operand::Temp) -> Result<i64, interpret::Error> {
        self.0.get(temp)
            .cloned()
            .ok_or_else(|| interpret::Error::UnboundTemp(*temp))
    }

    pub fn insert(&mut self, temp: operand::Temp, value: i64) {
        self.0.insert(temp, value);
    }
}
