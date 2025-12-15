use std::collections::HashMap;

use crate::common::binding::Binding;

use super::{error::CommonError, table::Table, value_type::ValueType};

pub enum SlabBindingInfo {
    Variable { idx: usize, value_type: ValueType },
    Function { idx: usize, ret_type: ValueType, arg_types: Vec<ValueType> },
}

pub struct Slab {
    data: Vec<usize>,
    hidden_state_count: usize,
    binding_map: HashMap<String, SlabBindingInfo>,
}

impl Slab {
    pub fn from_table(table: &Table) -> Result<Self, CommonError> {
        let mut binding_map = HashMap::<String, SlabBindingInfo>::new();
        let hidden_state_count = table.get_hidden_state_count();
        let mut idx = hidden_state_count;

        for (name, binding) in table.iter_bindings() {
           let info = match binding {
                Binding::Const { .. } |
                Binding::Function { .. } => None,
                Binding::Variable { value_type } => Some(SlabBindingInfo::Variable { idx, value_type: *value_type }),
            };

            if let Some(info) = info {
                binding_map.insert(name.clone(), info);
                idx += 1;
            }
        }

        let mut data = Vec::<usize>::with_capacity(idx);
        // XXX: the data in the vector could literally be anything, so we might
        //      as well just set the length and leave it uninitialized. it's the
        //      user's responsibility to set each value before evaluating the
        //      expression using this slab
        unsafe { data.set_len(idx); }

        Ok(Slab { data, hidden_state_count, binding_map })
    }

    pub fn get_binding_info(&self, name: &String) -> Option<&SlabBindingInfo> {
        self.binding_map.get(name)
    }

    pub fn get_binding_index(&self, name: &String) -> Option<usize> {
        match self.binding_map.get(name) {
            Some(info) => Some(match info {
                SlabBindingInfo::Variable { idx, .. } |
                SlabBindingInfo::Function { idx, .. } => *idx,
            }),
            None => None,
        }
    }

    pub fn get_hidden_state_count(&self) -> usize {
        self.hidden_state_count
    }

    pub fn get_address(&self, idx: usize) -> usize {
        unsafe { self.data.as_ptr().offset(idx.try_into().unwrap()).addr() }
    }

    #[inline(always)]
    pub fn set_value<T>(&mut self, idx: usize, value: T) {
        debug_assert!(idx < self.data.len());
        unsafe { *(self.data.as_mut_ptr().offset(idx as isize) as *mut T) = value }
    }

    #[inline(always)]
    pub fn set_ptr_value<T>(&mut self, idx: usize, value: *const T) {
        self.data[idx] = value.addr();
    }

    pub fn get_value<T: Copy>(&self, idx: usize) -> T {
        unsafe { *(self.data.as_ptr().offset(idx as isize) as *const T) }
    }
}