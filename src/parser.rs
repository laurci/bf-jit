use std::str::Chars;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    Right(u32),
    Left(u32),
    Increment(u32),
    Decrement(u32),
    Output,
    Input,
    Loop(Vec<Operation>),
}

pub fn parse_input(input: &mut Chars<'_>) -> Vec<Operation> {
    let mut ops = vec![];

    loop {
        let Some(c) = input.next() else {
            break;
        };

        match c {
            '<' => ops.push(Operation::Left(1)),
            '>' => ops.push(Operation::Right(1)),
            '+' => ops.push(Operation::Increment(1)),
            '-' => ops.push(Operation::Decrement(1)),
            '.' => ops.push(Operation::Output),
            ',' => ops.push(Operation::Input),
            '[' => {
                let inner = parse_input(input);
                ops.push(Operation::Loop(inner));
            }
            ']' => break,
            _ => {}
        };
    }

    ops
}

pub fn parse_and_optimize_input(input: &str) -> Vec<Operation> {
    let mut input = input.chars();
    let ops = parse_input(&mut input);
    optimize_ops(ops)
}

pub fn optimize_ops(ops: Vec<Operation>) -> Vec<Operation> {
    let mut optimized = vec![];

    let mut iter = ops.iter().peekable();

    loop {
        let Some(op) = iter.next() else {
            break;
        };

        match op {
            Operation::Right(n) => {
                let mut count = *n;

                while let Some(Operation::Right(n)) = iter.peek() {
                    count += n;
                    iter.next();
                }

                optimized.push(Operation::Right(count));
            }
            Operation::Left(n) => {
                let mut count = *n;

                while let Some(Operation::Left(n)) = iter.peek() {
                    count += n;
                    iter.next();
                }

                optimized.push(Operation::Left(count));
            }
            Operation::Increment(n) => {
                let mut count = *n;

                while let Some(Operation::Increment(n)) = iter.peek() {
                    count += n;
                    iter.next();
                }

                optimized.push(Operation::Increment(count));
            }
            Operation::Decrement(n) => {
                let mut count = *n;

                while let Some(Operation::Decrement(n)) = iter.peek() {
                    count += n;
                    iter.next();
                }

                optimized.push(Operation::Decrement(count));
            }
            Operation::Loop(inner_ops) => {
                let inner_ops = optimize_ops(inner_ops.clone());
                optimized.push(Operation::Loop(inner_ops));
            }
            _ => {
                optimized.push(op.clone());
            }
        }
    }

    optimized
}
