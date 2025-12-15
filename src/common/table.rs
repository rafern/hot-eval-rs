use std::collections::{HashMap, hash_map::Iter};

use super::{binding::Binding, error::CommonError, value_type::ValueType};

pub struct Table {
    bindings: HashMap<String, Binding>,
    hidden_states: Vec<ValueType>,
}

impl Table {
    pub fn new() -> Self {
        Table { bindings: HashMap::new(), hidden_states: Vec::new() }
    }

    pub fn add_binding(&mut self, name: String, binding: Binding) -> Result<(), CommonError> {
        if self.bindings.contains_key(&name) {
            Err(CommonError::BindingAlreadyExists { name })
        } else {
            self.bindings.insert(name, binding);
            Ok(())
        }
    }

    pub fn get_binding<'table>(&'table self, name: &String) -> Option<&'table Binding> {
        self.bindings.get(name)
    }

    pub fn iter_bindings(&self) -> Iter<'_, String, Binding> {
        self.bindings.iter()
    }

    pub fn add_hidden_state(&mut self, value_type: ValueType) -> usize {
        self.hidden_states.push(value_type);
        self.hidden_states.len() - 1
    }

    pub fn get_hidden_state<'table>(&'table self, hidden_state_idx: usize) -> Option<&'table ValueType> {
        self.hidden_states.get(hidden_state_idx)
    }

    pub fn get_hidden_state_count(&self) -> usize {
        self.hidden_states.len()
    }
}