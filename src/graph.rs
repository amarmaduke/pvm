use std::collections::{HashMap, HashSet};
use std::cmp::min;
use machine;

pub struct CallMetadata {
    pub is_left_call : bool,
    pub is_cyclic : bool,
    pub jump_position : isize
}

pub fn metadata(program : &Vec<machine::Instruction>) -> HashMap<usize, CallMetadata> {
    use machine::Instruction::*;

    let jump_table = {
        let mut result = (0..program.len()).map(|_| -1).collect::<Vec<isize>>();
        let mut current : isize = -1;
        
        for i in (0..program.len()).rev() {
            match program[i] {
                Return => current = i as isize,
                _ => { }
            }
            result[i] = current;
        }
        result
    };

    let call_table = {
        let mut result = vec![];
        let mut current = 0;

        for instruction in program.iter() {
            if &Return == instruction || &Stop == instruction {
                current += 1;
            }
            
            result.push(current);
        }
        result
    };

    let mut call_metadata = {
        let mut result = HashMap::new();
        let mut i = 0;
        let mut is_left_call = true;

        for instruction in program.iter() {
            match instruction {
                &Char(_)
                | &TestChar(_, _)
                | &Any
                | &TestAny(_, _)
                | &CharRange(_, _)
                | &CharRangeLink(_, _, _)
                => is_left_call = false,
                &Return | &Stop => is_left_call = true,
                &Call(_) | &PrecedenceCall(_, _) => {
                    result.insert(i, CallMetadata {
                        is_left_call: is_left_call,
                        is_cyclic: false,
                        jump_position: jump_table[i]
                    });
                },
                _ => { }
            }
            i += 1;
        }
        result
    };

    let graph = {
        let mut result = vec![];

        let limit = call_table.iter().max().unwrap_or(&0) + 1;
        for _ in 0..limit {
            result.push(vec![]);
        }

        let mut i = 0;
        for instruction in program.iter() {
            match instruction {
                &Call(j) | &PrecedenceCall(j, _) => {
                    let head = call_table[i];
                    let tail = call_table[(i as isize + j) as usize];
                    result[head].push((tail, i));
                },
                _ => { }
            }
            i += 1;
        }
        result
    };

    let cycles = detect_cycles(&graph);

    for i in cycles {
        if let Some(ref mut x) = call_metadata.get_mut(&i) {
            x.is_cyclic = true;
        }
    }

    call_metadata
}

fn detect_cycles(graph : &Vec<Vec<(usize, usize)>>) -> Vec<usize> {
    let mut stack = vec![];
    let mut stack_test = (0..graph.len()).map(|_| false).collect::<Vec<bool>>();
    let mut indices = (0..graph.len()).map(|_| -1).collect::<Vec<i32>>();
    let mut low_link = (0..graph.len()).map(|_| -1).collect::<Vec<i32>>();
    let mut cyclic_calls = HashSet::new();
    let mut index = 0;

    for v in 0..graph.len() {
        if indices[v] == -1 {
            strong_connect(v,
                &mut stack,
                &mut stack_test,
                &mut indices,
                &mut low_link,
                &mut index,
                &graph,
                &mut cyclic_calls);
        }
    }

    let result = cyclic_calls.drain().collect();
    result
}

fn strong_connect(vertex : usize,
    mut stack : &mut Vec<usize>,
    mut stack_test : &mut Vec<bool>,
    mut indices : &mut Vec<i32>,
    mut low_link : &mut Vec<i32>,
    mut index : &mut i32,
    graph : &Vec<Vec<(usize, usize)>>,
    mut cyclic_calls : &mut HashSet<usize>)
{
    indices[vertex] = *index;
    low_link[vertex] = *index;
    *index += 1;
    stack.push(vertex);
    stack_test[vertex] = true;

    for edge in graph[vertex].iter() {
        let tail = edge.0;
        if indices[tail] == -1 {
            strong_connect(tail,
                &mut stack,
                &mut stack_test,
                &mut indices,
                &mut low_link,
                &mut index,
                &graph,
                &mut cyclic_calls);
            low_link[vertex] = min(low_link[vertex], low_link[tail]);
        } else if stack_test[tail] {
            low_link[vertex] = min(low_link[vertex], indices[tail]);
        }
    }

    if low_link[vertex] == indices[vertex] {
        let mut strongly_connected_set = HashSet::new();

        while let Some(v) = stack.pop() {
            stack_test[v] = false;
            strongly_connected_set.insert(v);
            if v == vertex { break; }
        }

        for v in strongly_connected_set.iter() {
            for e in graph[*v].iter() {
                if strongly_connected_set.contains(&e.0) {
                    cyclic_calls.insert(e.1);
                }
            }
        }
    }
}
