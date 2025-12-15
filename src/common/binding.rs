use super::{value::Value, value_type::ValueType};

pub enum BindingFunctionParameter {
    Parameter { value_type: ValueType },
    ConstArgument { value: Value },
    HiddenStateArgument { hidden_state_idx: usize, cast_to_type: Option<ValueType> },
}

pub enum Binding {
    Const { value: Value },
    Variable { value_type: ValueType },
    Function { ret_type: ValueType, params: Vec<BindingFunctionParameter>, fn_ptr: *const () },
}