

enum StackFrame {
   Return(usize),
   Backtrack(usize, usize)
}

pub enum Instruction {
   Char(u8),
   Any,
   Choice(usize),
   Jump(usize),
   Call(usize),
   Return,
   Commit(usize),
   Fail
}

pub struct Machine {
   program : Vec<Instruction>,
   stack : Vec<StackFrame>,
   pc : usize,
   i : usize,
   fail : bool
}

impl Machine {
   pub fn execute(&mut self, input : Vec<u8>) {
      self.stack.clear();

      while self.i < input.len() {
         if self.fail {
            if let Some(frame) = self.stack.pop() {
               if let StackFrame::Backtrack(ret, j) = frame {
                  self.pc = ret;
                  self.i = j;
                  self.fail = false;
               }
            } else { break; }
         } else {
            match self.program[self.pc] {
               Instruction::Char(c) => {
                  if input[self.i] == c {
                     self.pc += 1;
                     self.i += 1;
                  } else {
                     self.fail = true;
                  }
               },
               Instruction::Any => {
                  if self.i + 1 < input.len() {
                     self.pc += 1;
                     self.i += 1;
                  } else {
                     self.fail = true;
                  }
               },
               Instruction::Choice(j) => {
                  self.stack.push(StackFrame::Backtrack(self.pc + j, self.i));
                  self.pc += 1;
               },
               Instruction::Jump(j) => {
                  self.pc += j;
               },
               Instruction::Call(j) => {
                  self.stack.push(StackFrame::Return(self.pc + 1));
                  self.pc += j;
               },
               Instruction::Return => {
                  if let Some(frame) = self.stack.pop() {
                     if let StackFrame::Return(ret) = frame {
                        self.pc = ret;
                     }
                  }
               },
               Instruction::Commit(j) => {
                  self.stack.pop();
                  self.pc += j;
               },
               Instruction::Fail => {
                  self.fail = true;
               }
            }
         }
      }
   }

   pub fn new(program : Vec<Instruction>) -> Machine {
      Machine {
         program: program,
         stack: vec![],
         pc: 0,
         i: 0,
         fail: false
      }
   }
}
