use std::collections::{HashMap, HashSet, BTreeMap, BTreeSet};
use std::cmp::{min, max};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VirtualRegister(pub u32);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PhysicalRegister(pub u8);

#[derive(Debug, Clone)]
pub struct Interval {
    pub start: u32,
    pub end: u32,
    pub register: Option<Register>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Register {
    Virtual(VirtualRegister),
    Physical(PhysicalRegister),
}

#[derive(Debug, Default)]
pub struct RegisterAllocator {
    virtual_registers: HashMap<VirtualRegister, Interval>,
    interference_graph: HashMap<VirtualRegister, HashSet<VirtualRegister>>,
    allocation: HashMap<VirtualRegister, PhysicalRegister>,
    stack_slots: HashMap<VirtualRegister, u32>,
    colors: Vec<bool>,
    num_physical: u8,
    next_virtual: u32,
    next_stack_slot: u32,
}

impl RegisterAllocator {
    pub fn new(num_physical: u8) -> Self {
        Self {
            virtual_registers: HashMap::new(),
            interference_graph: HashMap::new(),
            allocation: HashMap::new(),
            stack_slots: HashMap::new(),
            colors: vec![false; num_physical as usize],
            num_physical,
            next_virtual: 0,
            next_stack_slot: 0,
        }
    }

    pub fn new_virtual(&mut self) -> VirtualRegister {
        let reg = VirtualRegister(self.next_virtual);
        self.next_virtual += 1;
        reg
    }

    pub fn add_interval(&mut self, reg: VirtualRegister, start: u32, end: u32) {
        self.virtual_registers.insert(reg.clone(), Interval {
            start,
            end,
            register: Some(Register::Virtual(reg.clone())),
        });
        
        self.interference_graph.entry(reg.clone()).or_insert_with(HashSet::new);
    }

    pub fn build_interference_graph(&mut self) {
        for (reg1, &Interval { start: s1, end: e1, .. }) in &self.virtual_registers {
            for (reg2, &Interval { start: s2, end: e2, .. }) in &self.virtual_registers {
                if reg1 != reg2 && self.intervals_overlap(s1, e1, s2, e2) {
                    self.interference_graph.get_mut(reg1).unwrap().insert(reg2.clone());
                    self.interference_graph.get_mut(reg2).unwrap().insert(reg1.clone());
                }
            }
        }
    }

    fn intervals_overlap(&self, s1: u32, e1: u32, s2: u32, e2: u32) -> bool {
        !(e1 < s2 || e2 < s1)
    }

    pub fn allocate_registers(&mut self) -> Result<(), AllocationError> {
        let mut colored: HashSet<VirtualRegister> = HashSet::new();
        let mut spilled: HashSet<VirtualRegister> = HashSet::new();
        
        let mut worklist: BTreeSet<VirtualRegister> = self.virtual_registers.keys().cloned().collect();
        let mut initial_worklist = worklist.clone();
        
        while let Some(reg) = self.select_colorable(&mut worklist, &colored, &spilled) {
            if self.degree(&reg) < self.num_physical as usize {
                colored.insert(reg.clone());
                worklist.remove(&reg);
                
                for neighbor in self.interference_graph.get(&reg).unwrap().clone() {
                    worklist.insert(neighbor);
                }
            } else {
                spilled.insert(reg.clone());
                worklist.remove(&reg);
            }
        }

        for reg in initial_worklist {
            if !colored.contains(&reg) && !spilled.contains(&reg) {
                if self.degree(&reg) >= self.num_physical as usize {
                    spilled.insert(reg);
                }
            }
        }

        for reg in colored {
            let color = self.find_color(&reg, &colored);
            self.allocation.insert(reg, PhysicalRegister(color as u8));
        }

        for reg in spilled {
            let slot = self.next_stack_slot;
            self.next_stack_slot += 1;
            self.stack_slots.insert(reg, slot);
        }

        if !spilled.is_empty() {
            self.resolve_spills(&spilled);
        }

        Ok(())
    }

    fn select_colorable(
        &self,
        worklist: &mut BTreeSet<VirtualRegister>,
        _colored: &HashSet<VirtualRegister>,
        _spilled: &HashSet<VirtualRegister>,
    ) -> Option<VirtualRegister> {
        worklist.iter().min_by_key(|r| self.degree(r)).cloned()
    }

    fn degree(&self, reg: &VirtualRegister) -> usize {
        self.interference_graph.get(reg)
            .map(|set| set.len())
            .unwrap_or(0)
    }

    fn find_color(&self, reg: &VirtualRegister, colored: &HashSet<VirtualRegister>) -> usize {
        self.colors.fill(false);
        
        if let Some(neighbors) = self.interference_graph.get(reg) {
            for neighbor in neighbors {
                if let Some(&PhysicalRegister(c)) = self.allocation.get(neighbor) {
                    if c < self.num_physical {
                        self.colors[c as usize] = true;
                    }
                }
            }
        }
        
        for i in 0..self.num_physical as usize {
            if !self.colors[i] {
                return i;
            }
        }
        
        0
    }

    fn resolve_spills(&mut self, _spilled: &HashSet<VirtualRegister>) {
    }

    pub fn get_allocation(&self, reg: &VirtualRegister) -> Option<PhysicalRegister> {
        self.allocation.get(reg).copied()
    }

    pub fn get_stack_slot(&self, reg: &VirtualRegister) -> Option<u32> {
        self.stack_slots.get(reg).copied()
    }

    pub fn is_spilled(&self, reg: &VirtualRegister) -> bool {
        self.stack_slots.contains_key(reg)
    }
}

#[derive(Debug)]
pub struct AllocationError {
    pub message: String,
}

pub struct LivenessInfo {
    pub use_positions: HashMap<VirtualRegister, Vec<u32>>,
    pub def_positions: HashMap<VirtualRegister, Vec<u32>>,
    pub live_ranges: HashMap<VirtualRegister, (u32, u32)>,
}

impl LivenessInfo {
    pub fn new() -> Self {
        Self {
            use_positions: HashMap::new(),
            def_positions: HashMap::new(),
            live_ranges: HashMap::new(),
        }
    }

    pub fn compute_live_ranges(&mut self) {
        for (reg, uses) in &self.use_positions {
            let defs = self.def_positions.get(reg).unwrap_or(&vec![]);
            let start = defs.first().copied().unwrap_or(0);
            let end = uses.last().copied().unwrap_or(0);
            self.live_ranges.insert(reg.clone(), (start, end));
        }
    }
}

pub fn compute_liveness(cfg: &crate::ControlFlowGraph) -> LivenessInfo {
    let mut info = LivenessInfo::new();
    info
}

pub fn allocate_registers(
    cfg: &crate::ControlFlowGraph,
    machine: &crate::MachineTarget,
) -> Result<RegisterAllocation, AllocationError> {
    let mut allocator = RegisterAllocator::new(machine.num_registers());
    let mut liveness = LivenessInfo::new();
    
    liveness.compute_live_ranges();
    
    for (reg, &(start, end)) in &liveness.live_ranges {
        allocator.add_interval(reg.clone(), start, end);
    }
    
    allocator.build_interference_graph();
    allocator.allocate_registers()?;
    
    Ok(RegisterAllocation {
        allocation: allocator.allocation,
        stack_slots: allocator.stack_slots,
    })
}

#[derive(Debug)]
pub struct RegisterAllocation {
    pub allocation: HashMap<VirtualRegister, PhysicalRegister>,
    pub stack_slots: HashMap<VirtualRegister, u32>,
}
