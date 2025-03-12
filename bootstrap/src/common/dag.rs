use std::collections::VecDeque;

use bootstrap_macros::flags;



pub struct DagNode<NodeData> {
    predecessors:  Vec<u32>,
    data:          NodeData,
    precomp_preds: Vec<u32>,
}

pub struct Dag<NodeData> {
    nodes: Vec<DagNode<NodeData>>,
}

#[allow(unused)]
impl<T> Dag<T> {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
        }
    }

    pub fn add_node(&mut self, data: T) -> u32 {
        let idx = self.nodes.len();
        self.nodes.push(DagNode {
            predecessors: Vec::new(),
            data,
            precomp_preds: Vec::new(),
        });
        idx as u32
    }

    pub fn set_predecessor(&mut self, idx: u32, predecessor: u32) {
        assert!((idx as usize) < self.nodes.len());
        assert!((predecessor as usize) < self.nodes.len());

        self.nodes[idx as usize].predecessors.push(predecessor);
    }

    pub fn get_data(&self, idx: u32) -> Option<&T> {
        let idx = idx as usize;
        if idx < self.nodes.len() {
            Some(&self.nodes[idx].data)
        } else {
            None
        }
    }

    pub fn get_data_mut(&mut self, idx: u32) -> Option<&mut T> {
        let idx = idx as usize;
        if idx < self.nodes.len() {
            Some(&mut self.nodes[idx].data)
        } else {
            None
        }
    }

    pub fn find<F>(&self, mut f: F) -> Option<(u32, &T)> where 
        F: FnMut(u32, &T) -> bool
    {
        self.nodes.iter().enumerate()
            .find(|(idx, node)| f(*idx as u32, &node.data))
            .map(|(idx, node)| (idx as u32, &node.data))
    }

    pub fn find_mut<F>(&mut self, mut f: F) -> Option<(u32, &mut T)> where 
        F: FnMut(u32, &T) -> bool
    {
        self.nodes.iter_mut().enumerate()
            .find(|(idx, node)| f(*idx as u32, &node.data))
            .map(|(idx, node)| (idx as u32, &mut node.data))
    }

    pub fn find_map<F, Ret>(&self, mut f: F) -> Option<Ret> where 
        F: FnMut(u32, &T) -> Option<Ret>
    {
        self.nodes.iter().enumerate()
            .find_map(|(idx, node)| f(idx as u32, &node.data))
    }

    pub fn has_predecessor(&self, idx: u32, predecessor: u32) -> bool {
        assert!((idx as usize) < self.nodes.len());
        assert!((predecessor as usize) < self.nodes.len());

        self.nodes[idx as usize].precomp_preds.contains(&predecessor)
    }

    pub fn calculate_predecessors(&mut self) {
        let mut processed_flag = Vec::new();
        let mut to_process = VecDeque::new();

        processed_flag.resize(self.nodes.len(), false);

        to_process.reserve(self.nodes.len());
        for idx in 0..self.nodes.len() {
            to_process.push_back(idx);
        }

        'outer: while let Some(idx) = to_process.pop_front() {
            let node = &self.nodes[idx];
            if node.predecessors.is_empty() {
                processed_flag[idx] = true;
                continue;
            }

            // Check if all predecessor nodes have been processed
            for pred in &node.predecessors {
                if !processed_flag[*pred as usize] {
                    to_process.push_back(idx);
                    continue 'outer;
                }
            }

            // All are processed, so combine them now
            let mut all_preds = Vec::new();
            for pred in &node.predecessors {
                all_preds.push(*pred);

                let pred = &self.nodes[*pred as usize];
                all_preds.extend_from_slice(&pred.precomp_preds);
            }

            all_preds.dedup();
            all_preds.sort();

            self.nodes[idx].precomp_preds = all_preds;
            processed_flag[idx] = true;
        }
    }

    pub fn check_cycles(&self) -> Vec<Vec<u32>> {
        #[flags]
        enum ProcessState {
            Visited,
            Finished,
            Cycle,
        }
        let mut states = Vec::new();
        states.resize(self.nodes.len(), ProcessState::None);

        let mut full_cycles = Vec::new();

        let mut to_check = Vec::new();

        // Try a path from each node
        for idx in 0..self.nodes.len() {
            // If we already fully checked this node in a previous iteration, skip it
            if states[idx].intersects(ProcessState::Visited | ProcessState::Cycle) {
                continue;
            }

            let node = &self.nodes[idx];

            // If we don't have any predecessors, we can't have a cycle from this node, so skip
            if node.predecessors.is_empty() {
                states[idx] = ProcessState::Finished;
                continue;
            }

            // Add all possible path starts here
            for pred in &node.predecessors {
                to_check.push((*pred, false));
            }

            states[idx] = ProcessState::Visited;

            // Loop through all possible paths we still need to check
            // We don't check if we move into a new cycle, as we don't want to mark a path into a cycle as the source of a cycle
            let mut has_cycle = false;
            while let Some((sub_idx, needs_pop)) = to_check.pop() {
                // Special case, added in each path so we know we finished processing a sub-path without needing recursion
                let sub_idx = sub_idx as usize;
                if needs_pop {
                    states[sub_idx] = if has_cycle {
                        ProcessState::Cycle
                    } else {
                        ProcessState::Finished
                    };
                    continue;
                }

                // If this node was already finished, we cannot have a new cycle starting from this point on
                if states[sub_idx].intersects(ProcessState::Finished) {
                    continue;
                }

                // Oops, we've been here already, so we have a cycle
                if states[sub_idx] == ProcessState::Visited {
                    states[sub_idx] = ProcessState::Cycle;
                    has_cycle = true;

                    // Get the full cycle for error reporting later by going over all to_check nodes,
                    // finding the ones that are refs to previous nodes in the chain,
                    // and following them until we hit the hit node
                    let mut cycle = vec![sub_idx as u32];
                    for (id, pop) in to_check.iter().rev() {
                        if !pop {
                            continue;
                        }

                        // We hit out current node, so we're back at the start
                        if *id as usize == sub_idx {
                            break;
                        }
                        cycle.push(*id);
                    }
                    full_cycles.push(cycle);

                    continue;
                }

                // Set up the current node as visited and requiring a pop later on
                // Then add all predecessors for this node to check
                states[sub_idx] = ProcessState::Visited;
                to_check.push((sub_idx as u32, true));
                for pred in &self.nodes[sub_idx].predecessors {
                    to_check.push((*pred, false));
                }
            }

            // Set finished if this is not a node in a cycle, 
            // so we can skip any new node that would enter this node that leads to a cycle, but is not part of it
            states[idx] = if has_cycle {
                ProcessState::Cycle
            } else {
                ProcessState::Finished
            };
        }

        // Post-process all cycles and and return all full cycles

        // 1. Change all cycles so that the smallest index is at the start
        for cycle in &mut full_cycles {
            let mut lowest = (0, u32::MAX);
            for (idx, val) in cycle.iter().enumerate() {
                if *val < lowest.1 {
                    lowest = (idx, *val);
                }
            }

            if lowest.0 != 0 {
                let new_head = cycle.split_off(lowest.0);
                let mut new_tail = std::mem::replace(cycle, new_head);
                cycle.append(&mut new_tail);
            }
        }

        // 2. dedup all cycles
        full_cycles.dedup();

        full_cycles
    }

    pub fn get_precomputed_predecessor_idxs(&self, idx: u32) -> &[u32] {
        &self.nodes[idx as usize].precomp_preds
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter { iter: self.nodes.iter() }
    }

    pub fn iter_mut<'a>(&'a mut self) -> IterMut<'a, T> {
        IterMut { iter: self.nodes.iter_mut() }
    }
}

pub struct Iter<'a, T: 'a> {
    iter: std::slice::Iter<'a, DagNode<T>>
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|node| &node.data)
    }
}
pub struct IterMut<'a, T: 'a> {
    iter: std::slice::IterMut<'a, DagNode<T>>
}

impl<'a, T: 'a> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|node| &mut node.data)
    }
}