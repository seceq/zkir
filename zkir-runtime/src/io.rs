//! I/O handling

use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct IOHandler {
    inputs: VecDeque<u32>,
    hints: VecDeque<u32>,
    outputs: Vec<u32>,
    commitments: Vec<u32>,
}

impl IOHandler {
    pub fn new(inputs: Vec<u32>) -> Self {
        IOHandler {
            inputs: inputs.into(),
            hints: VecDeque::new(),
            outputs: Vec::new(),
            commitments: Vec::new(),
        }
    }

    pub fn read(&mut self) -> Option<u32> {
        self.inputs.pop_front()
    }

    pub fn read_hint(&mut self) -> Option<u32> {
        self.hints.pop_front()
    }

    pub fn write(&mut self, value: u32) {
        self.outputs.push(value);
    }

    pub fn commit(&mut self, value: u32) {
        self.commitments.push(value);
    }

    pub fn outputs(&self) -> &[u32] {
        &self.outputs
    }

    pub fn commitments(&self) -> &[u32] {
        &self.commitments
    }

    pub fn take_outputs(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.outputs)
    }

    pub fn take_commitments(&mut self) -> Vec<u32> {
        std::mem::take(&mut self.commitments)
    }

    pub fn inputs(&mut self) -> &[u32] {
        self.inputs.make_contiguous()
    }
}
