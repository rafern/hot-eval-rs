use std::{collections::HashMap, mem::MaybeUninit};

use crate::common::binding::Binding;

use super::{error::CommonError, table::Table, value_type::ValueType};

pub enum SlabBindingInfo {
    Variable { idx: usize, value_type: ValueType },
    Function { idx: usize, ret_type: ValueType, arg_types: Vec<ValueType> },
}

pub struct Slab {
    data: Box<[MaybeUninit<usize>]>,
    hidden_state_count: usize,
    binding_map: HashMap<String, SlabBindingInfo>,
    hidden_state_types: Box<[ValueType]>,
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

        let mut hst = Vec::new();
        for i in 0..hidden_state_count {
            hst.push(*table.get_hidden_state(i).unwrap());
        }

        let mut data = Vec::with_capacity(idx);
        data.resize(idx, MaybeUninit::new(0));

        Ok(Slab { data: data.into(), hidden_state_count, binding_map, hidden_state_types: hst.into() })
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

    pub const fn get_hidden_state_count(&self) -> usize {
        self.hidden_state_count
    }

    pub fn get_hidden_state_type(&self, idx: usize) -> Option<ValueType> {
        if idx < self.hidden_state_types.len() {
            Some(self.hidden_state_types[idx])
        } else {
            None
        }
    }

    pub fn get_address(&self, idx: usize) -> usize {
        (&self.data[idx] as *const MaybeUninit<usize>).addr()
    }

    #[inline(always)]
    const unsafe fn get_value_ptr_unchecked<T: Copy>(&mut self, idx: usize) -> *mut T {
        const { assert!(size_of::<T>() <= size_of::<usize>()); }
        // SAFETY: idx must be guaranteed to be < self.data.len() by the caller,
        //         so there is no OOB access, and T fits in a usize, so the
        //         pointer can be safely cast to *mut T
        unsafe { self.data.as_mut_ptr().offset(idx as isize) as *mut T }
    }

    #[inline(always)]
    pub const unsafe fn set_value_unchecked<T: Copy>(&mut self, idx: usize, value: T) {
        // SAFETY: idx must be guaranteed to be < self.data.len() by the caller
        unsafe { *self.get_value_ptr_unchecked(idx) = value };
    }

    #[inline(always)]
    pub const fn set_value<T: Copy>(&mut self, idx: usize, value: T) {
        assert!(idx < self.data.len());
        // SAFETY: idx < self.data.len(), so there is no OOB access
        unsafe { self.set_value_unchecked(idx, value) };
    }

    /// SAFETY: The caller must guarantee that the pointer is valid when the
    ///         expression using this Slab is evaluated
    #[inline(always)]
    pub unsafe fn set_ptr_value_unchecked<T>(&mut self, idx: usize, pointer: *const T) {
        // SAFETY: idx must be guaranteed to be < self.data.len() by the caller
        unsafe { self.set_value_unchecked(idx, pointer.addr()) };
    }

    /// SAFETY: The caller must guarantee that the pointer is valid when the
    ///         expression using this Slab is evaluated
    ///
    /// This method is unsafe since it only checks that idx is valid
    #[inline(always)]
    pub unsafe fn set_ptr_value<T>(&mut self, idx: usize, pointer: *const T) {
        self.set_value(idx, pointer.addr());
    }

    #[inline(always)]
    pub const unsafe fn get_value_unchecked<T: Copy>(&self, idx: usize) -> T {
        const { assert!(size_of::<T>() <= size_of::<usize>()); }
        // SAFETY: idx must be guaranteed to be < self.data.len() by the caller,
        //         so there is no OOB access, and T fits in a usize, so the
        //         pointer can be safely cast to *mut T
        unsafe { *(self.data.as_ptr().offset(idx as isize) as *const T) }
    }

    #[inline(always)]
    pub const fn get_value<T: Copy>(&self, idx: usize) -> T {
        assert!(idx < self.data.len());
        // SAFETY: idx < self.data.len(), so there is no OOB access
        unsafe { self.get_value_unchecked(idx) }
    }
}