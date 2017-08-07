//! A `ControlFlowGraph` is a directed `Graph` of `Block` and `Edge`.

use std::cell::Cell;
use std::collections::{BTreeMap};
use std::fmt;
use il::*;


/// Edge between IL blocks
///
/// A Falcon IL `Edge` has an optional condition. When the condition is present, the `Edge` is,
/// "Guarded," by the `Expression` in the condition. `Edge` conditions are `Expressions` that must
/// evaluate to a 1-bit `Constant`. When the condition evaluates to 1, the `Edge` may be taken.
/// Otherwise the `Edge` is not taken. When the condition is not present, the `Edge` is
/// unconditional and will always be taken.
///
/// To create a new edge, call `ControlFlowGraph::unconditional_edge` or
/// `ControlFlowGraph::conditional_edge`.

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Edge {
    head: u64,
    tail: u64,
    condition: Option<Expression>,
    comment: Option<String>
}


impl Edge {
    pub(crate) fn new(head: u64, tail: u64, condition: Option<Expression>) -> Edge {
        Edge {
            head: head,
            tail: tail,
            condition: condition,
            comment: None
        }
    }

    /// Retrieve the condition for this `Edge`.
    pub fn condition(&self) -> &Option<Expression> {
        &self.condition
    }

    /// Retrieve a mutable reference to the condition for this `Edge`
    pub fn condition_mut(&mut self) -> &mut Option<Expression> {
        &mut self.condition
    }

    /// Retrieve the index of the head `Vertex` for this `Edge`.
    pub fn head(&self) -> u64 { self.head }

    /// Retrieve the index of the tail `Vertex` for this `Edge`.
    pub fn tail(&self) -> u64 { self.tail }

    /// Set the comment for this `Edge`.
    pub fn set_comment(&mut self, comment: Option<String>) {
        self.comment = comment;
    }

    /// Get the comment for this `Edge`.
    pub fn comment(&self) -> &Option<String> {
        &self.comment
    }
}


impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref comment) = self.comment {
            write!(f, "// {}\n", comment)?
        }
        if let Some(ref condition) = self.condition {
            write!(
                f,
                "(0x{:X}->0x{:X}) ? ({})",
                self.head,
                self.tail,
                condition
            )?
        }
        Ok(())
    }
}


impl graph::Edge for Edge {
    fn head(&self) -> u64 { self.head }
    fn tail(&self) -> u64 { self.tail }
    fn dot_label(&self) -> String { format!("{}", self) }
}


/// A directed graph of types `Block` and `Edge`.
///
/// # Entry and Exit
/// A `ControlFlowGraph` has an optional, "Entry," and an optional, "Exit." When these are
/// provided, certain convenience functions become available.
///
/// For example, when translating a native instruction to Falcon IL, it can be useful to consider
/// an instruction as its own `ControlFlowGraph`. `rep scasb` is a great example of when this
/// pattern is helpful. Instructions in a `Block` will have one entry, and one exit. Explicitly
/// declaring these makes merging `ControlFlowGraph`s easier.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ControlFlowGraph {
    // The internal graph used to store our blocks.
    graph: graph::Graph<Block, Edge>,
    // The next index to use when creating a basic block.
    next_index: u64,
    // The index for the next temp variable to create.
    next_temp_index: Cell<u64>,
    // An optional entry index for the graph.
    entry: Option<u64>,
    // An optional exit index for the graph.
    exit: Option<u64>,
    // True if SSA has been applied to the ControlFlowGraph
    ssa_form: bool,
}


impl ControlFlowGraph {
    pub fn new() -> ControlFlowGraph {
        ControlFlowGraph {
            graph: graph::Graph::new(),
            next_index: 0,
            next_temp_index: Cell::new(0),
            entry: None,
            exit: None,
            ssa_form: false,
        }
    }


    /// Returns the underlying graph
    pub fn graph(&self) -> &graph::Graph<Block, Edge> {
        &self.graph
    }

    /// Sets the entry point for this `ControlFlowGraph` to the given `Block` index.
    pub fn set_entry(&mut self, entry: u64) -> Result<()> {
        if self.graph.has_vertex(entry) {
            self.entry = Some(entry);
            return Ok(());
        }
        Err("Index does not exist for set_entry".into())
    }

    /// Sets the exit point for this `ControlFlowGraph` to the given `Block` index.
    pub fn set_exit(&mut self, exit: u64) -> Result<()> {
        if self.graph.has_vertex(exit) {
            self.exit = Some(exit);
            return Ok(());
        }
        Err("Index does not exist for set_exit".into())
    }

    /// Get the entry `Block` index for this `ControlFlowGraph`.
    pub fn entry(&self) -> Option<u64> {
        self.entry
    }

    /// Get the exit `Block` index for this `ControlFlowGraph`.
    pub fn exit(&self) -> Option<u64> {
        self.exit
    }

    /// Get a `Block` by index.
    pub fn block(&self, index: u64) -> Option<&Block> {
        self.graph.vertex(index)
    }

    /// Get a mutable reference to a `Block` by index.
    pub fn block_mut(&mut self, index: u64) -> Option<&mut Block> {
        self.graph.vertex_mut(index)
    }

    /// Get every `Block` in this `ControlFlowGraph`.
    pub fn blocks(&self) -> Vec<&Block> {
        self.graph.vertices()
    }

    /// Get a mutable reference to every `Block` in this `ControlFlowGraph`.
    pub fn blocks_mut(&mut self) -> Vec<&mut Block> {
        self.graph.vertices_mut()
    }

    /// Get an `Edge` by its head and tail `Block` indices.
    pub fn edge(&self, head: u64, tail: u64) -> Option<&Edge> {
        self.graph.edge(head, tail)
    }

    /// Get a mutable reference to an `Edge` by its head and tail `Block` indices.
    pub fn edge_mut(&mut self, head: u64, tail: u64) -> Option<&mut Edge> {
        self.graph.edge_mut(head, tail)
    }

    /// Get every `Edge` in thie `ControlFlowGraph`.
    pub fn edges(&self) -> Vec<&Edge> {
        self.graph.edges()
    }

    /// Get a mutable reference to every `Edge` in this `ControlFlowGraph`.
    pub fn edges_mut(&mut self) -> Vec<&mut Edge> {
        self.graph.edges_mut()
    }


    /// Sets the address for all instructions in this `ControlFlowGraph`.
    ///
    /// Useful for translators to set address information.
    pub fn set_address(&mut self, address: Option<u64>) {
        for mut block in self.blocks_mut() {
            for mut instruction in block.instructions_mut() {
                instruction.set_address(address);
            }
        }
    }


    /// Returns the entry block for this ControlFlowGraph
    pub fn entry_block(&self) -> Option<&Block> {
        if self.entry.is_none() {
            None
        }
        else {
            self.block(self.entry.unwrap())
        }
    }


    /// Generates a temporary scalar unique to this control flow graph.
    pub fn temp(&self, bits: usize) -> Scalar {
        let next_index = self.next_temp_index.get();
        self.next_temp_index.set(next_index + 1);
        Scalar::new(format!("temp_{}", next_index), bits)
    }


    /// Creates a new basic block, adds it to the graph, and returns it
    pub fn new_block(&mut self) -> Result<&mut Block> {
        let next_index = self.next_index;
        self.next_index += 1;
        let block = Block::new(next_index);
        self.graph.insert_vertex(block)?;
        Ok(self.graph.vertex_mut(next_index).unwrap())
    }


    /// Creates an unconditional edge from one block to another block
    pub fn unconditional_edge(&mut self, head: u64, tail: u64) -> Result<()> {
        let edge = Edge::new(head, tail, None);
        self.graph.insert_edge(edge)
    }


    /// Creates a conditional edge from one block to another block
    pub fn conditional_edge(&mut self, head: u64, tail: u64, condition: Expression) -> Result<()> {
        let edge = Edge::new(head, tail, Some(condition));
        self.graph.insert_edge(edge)
    }


    /// Merge `Block`s.
    ///
    /// When a `Block` as only one successor, and that successor has only one predecessor, we
    /// merge both into one `Block`. 
    pub fn merge(&mut self) -> Result<()> {
        loop {
            let mut merge_index = None;
            let mut successor_index = None;
            for block in self.blocks() {
                // check to see how many successors we have
                let successors = self.graph.edges_out(block.index()).unwrap();

                // if we do not have just one successor, we will not merge this block
                if successors.len() != 1 {
                    continue;
                }

                // If this successor has a condition, we will not merge this block
                if successors.first().unwrap().condition().is_some() {
                    continue;
                }

                // get the vertex for this successor
                let successor: u64 = match successors.first() {
                    Some(successor) => successor.tail(),
                    None => bail!("successor not found")
                };

                // get all predecessors for this successor
                let predecessors = self.graph.edges_in(successor).unwrap();

                // if this successor does not have exactly one predecessor, we
                // will not merge this block
                if predecessors.len() != 1 {
                    continue;
                }

                successor_index = Some(successor);
                merge_index = Some(block.index());
                break;
            }

            // we have a block to merge
            if let Some(merge_index) = merge_index {
                let successor_index = match successor_index {
                    Some(successor_index) => successor_index,
                    None => bail!("merge_index set, but not successor_index")
                };

                // merge the blocks
                let successor_block = self.graph
                                          .vertex(successor_index)
                                          .ok_or("Could not find block")?
                                          .clone();
                self.graph
                    .vertex_mut(merge_index)
                    .ok_or("Could not find block")?
                    .append(&successor_block);

                // all of successor's successors become merge_block's successors
                let mut new_edges = Vec::new();
                for edge in self.graph.edges_out(successor_index).unwrap() {
                    let head = merge_index;
                    let tail = edge.tail();
                    let condition = edge.condition().clone();
                    let edge = Edge::new(head, tail, condition);
                    new_edges.push(edge);
                }
                for edge in new_edges {
                    self.graph.insert_edge(edge)?;
                }

                // remove the block we just merged
                self.graph.remove_vertex(successor_index)?;
            } else {
                break;
            }
        }
        Ok(())
    }


    /// Appends a control flow graph to this control flow graph.
    ///
    /// In order for this to work, the entry and exit of boths graphs must be
    /// set, which should be the case for all conformant translators. You can
    /// also append to an empty ControlFlowGraph.
    pub fn append(&mut self, other: &ControlFlowGraph) -> Result<()> {
        let is_empty = match self.graph.num_vertices() {
            0 => true,
            _ => false
        };

        if !is_empty && (self.entry().is_none() || self.exit().is_none()) {
            return Err("entry/exit not set for dest ControlFlowGraph::append".into());
        }
        
        if other.entry().is_none() || other.exit().is_none() {
            return Err("entry/exit not set for src ControlFlowGraph::append".into());
        }

        // Bring in new blocks
        let mut block_map: BTreeMap<u64, u64> = BTreeMap::new();
        for block in other.graph().vertices() {
            // we need to clone the underlying block
            let new_block = block.clone_new_index(self.next_index);
            block_map.insert(block.index(), self.next_index);
            self.next_index += 1;
            self.graph.insert_vertex(new_block)?;
        }

        // Now set all new edges
        for edge in other.graph().edges() {
            let new_head: u64 = block_map[&edge.head()];
            let new_tail: u64 = block_map[&edge.tail()];
            let new_edge = Edge::new(new_head, new_tail, edge.condition().clone());
            self.graph.insert_edge(new_edge)?;
        }


        if is_empty {
            self.entry = Some(block_map[&other.entry().unwrap()]);
        }
        else {
            // Create an edge from the exit of this graph to the head of the other
            // graph
            let transition_edge = Edge::new(
                self.exit.unwrap(),
                block_map[&(other.entry().unwrap())],
                None
            );
            self.graph.insert_edge(transition_edge)?;
        }

        self.exit = Some(block_map[&other.exit().unwrap()]);

        Ok(())
    }

    /// Inserts a control flow graph into this control flow graph, and returns
    /// the entry and exit indices for inserted graph.
    ///
    /// Requires the graph being inserted to have entry set.
    ///
    /// This function causes the `ControlFlowGraph` to become disconnected.
    ///
    /// This function is useful for inserting multiple `ControlFlowGraph`s into
    /// one before adding all `Edge`s in a subsequent pass.
    ///
    /// # Warnings
    /// This invalidates the entry and exit of the control flow graph.
    pub fn insert(&mut self, other: &ControlFlowGraph) -> Result<(u64, u64)> {
        if other.entry().is_none() || other.exit().is_none() {
            bail!("entry/exit not set on control flow graph");
        }

        // our entry and exit our no longer valid
        self.entry = None;
        self.exit = None;

        // Options to store the other graph entry/exit indices
        let mut entry_index = None;
        let mut exit_index = None;

        // keep track of mapping between old indices and new indices
        let mut block_map: BTreeMap<u64, u64> = BTreeMap::new();

        // insert all the blocks
        for block in other.graph().vertices() {
            let new_block = block.clone_new_index(self.next_index);
            block_map.insert(block.index(), self.next_index);
            if block.index() == other.entry().unwrap() {
                entry_index = Some(self.next_index);
            }
            if block.index() == other.exit().unwrap() {
                exit_index = Some(self.next_index);
            }
            self.next_index += 1;
            self.graph.insert_vertex(new_block)?;
        }

        // insert edges
        for edge in other.graph().edges() {
            let new_head: u64 = block_map[&edge.head()];
            let new_tail: u64 = block_map[&edge.tail()];
            let new_edge = Edge::new(new_head, new_tail, edge.condition().clone());
            self.graph.insert_edge(new_edge)?;
        }

        if entry_index.is_none() || exit_index.is_none() {
            bail!("failed to get entry/exit indices");
        }

        Ok((entry_index.unwrap(), exit_index.unwrap()))
    }
}


impl fmt::Display for ControlFlowGraph {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for block in self.blocks() {
            let r = writeln!(f, "{}", block);
            if r.is_err() {
                return r;
            }
        }
        Ok(())
    }
}