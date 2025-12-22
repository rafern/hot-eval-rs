use std::collections::{HashMap, hash_map::Iter};

use super::{binding::{Binding, FnPointer, FnSpecCallArg, FnSpecChoice, ToBFPValueType}, error::CommonError, value::Value, value_type::ValueType};

struct BindingFunctionParamBuilder {
    mapping: HashMap<usize, ValueType>,
}

impl BindingFunctionParamBuilder {
    pub fn new() -> Self {
        Self { mapping: HashMap::new() }
    }

    pub fn maybe_add_spec_call_arg(&mut self, arg: &FnSpecCallArg, fn_ptr_arg_type: ValueType) -> Result<(), CommonError> {
        if let FnSpecCallArg::MappedArgument { param_idx } = arg {
            let idx = *param_idx;
            if let Some(existing_type) = self.mapping.insert(idx, fn_ptr_arg_type) && fn_ptr_arg_type != existing_type {
                return Err(CommonError::FuncSpecArgParamIndexConflict { idx, new_type: fn_ptr_arg_type, existing_type });
            }
        }

        Ok(())
    }

    pub fn finish(self) -> Result<Box<[ValueType]>, CommonError> {
        let mut params = Vec::new();
        let len = self.mapping.len();
        for i in 0..len {
            match self.mapping.get(&i) {
                Some(value_type) => params.push(*value_type),
                None => return Err(CommonError::FuncSpecArgDiscontinuousParamMap { max_idx: len - 1, missing_idx: i }),
            }
        }

        Ok(params.into())
    }
}

pub struct Table<'table> {
    bindings: HashMap<String, Binding<'table>>,
    hidden_states: Vec<ValueType>,
}

impl<'table> Table<'table> {
    pub fn new() -> Self {
        Table { bindings: HashMap::new(), hidden_states: Vec::new() }
    }

    pub unsafe fn add_binding(&mut self, name: String, binding: Binding<'table>) -> Result<(), CommonError> {
        if self.bindings.contains_key(&name) {
            Err(CommonError::BindingAlreadyExists { name })
        } else {
            self.bindings.insert(name, binding);
            Ok(())
        }
    }

    pub fn get_binding(&self, name: &String) -> Option<&'_ Binding<'_>> {
        self.bindings.get(name)
    }

    pub fn iter_bindings(&self) -> Iter<'_, String, Binding<'_>> {
        self.bindings.iter()
    }

    pub fn add_hidden_state(&mut self, value_type: ValueType) -> usize {
        self.hidden_states.push(value_type);
        self.hidden_states.len() - 1
    }

    pub fn add_ptr_hidden_state(&mut self) -> usize {
        self.add_hidden_state(ValueType::USize)
    }

    pub fn get_hidden_state(&'table self, hidden_state_idx: usize) -> Option<&'table ValueType> {
        self.hidden_states.get(hidden_state_idx)
    }

    pub fn get_hidden_state_count(&self) -> usize {
        self.hidden_states.len()
    }

    pub fn add_const<T: Into<Value>>(&mut self, name: String, value: T) -> Result<(), CommonError> {
        unsafe { self.add_binding(name, Binding::Const { value: value.into() }) }
    }

    pub fn add_variable(&mut self, name: String, value_type: ValueType) -> Result<(), CommonError> {
        unsafe { self.add_binding(name, Binding::Variable { value_type }) }
    }

    // TODO: having different methods for each number of parameters is a
    //       complete mess, but the alternative is using a trait like
    //       IntoBinding for fn pointers, where you still have to implement the
    //       trait for each number of parameters. the problem is that, even if
    //       that was an acceptable compromise (which it is), functions aren't
    //       implicitly coerced into fn pointers when using them as type
    //       parameters for traits, so you have to cast the function to a
    //       pointer. e.g.:
    //       table.add_binding("test", test);
    //       ... becomes:
    //       table.add_binding("test", test as fn() -> f64);
    //       this defeats the whole purpose of having these helper methods.
    //       it would also be nice if this code could somehow be reused into a
    //       single implementation, but that would require rust to have a more
    //       flexible type system. because of this, i'm only implementing up to
    //       a few parameters for my sanity. if you need more, then use the
    //       not-so-safe version where you have to pass a Binding value directly

    pub fn add_function_0<R>(&mut self, name: String, fn_ptr: fn() -> R) -> Result<(), CommonError>
    where
        R: ToBFPValueType,
    {
        let fn_ptr = fn_ptr as FnPointer;
        unsafe { self.add_binding(name, Binding::Function {
            ret_type: R::to_bfp_value_type(),
            params: [].into(),
            fn_spec: Box::new(move |_| Ok(FnSpecChoice::Call { fn_ptr, args: [].into() })),
        }) }
    }

    pub fn add_function_1<R, P1>(&mut self, name: String, fn_ptr: fn(P1) -> R) -> Result<(), CommonError>
    where
        R: ToBFPValueType,
        P1: ToBFPValueType,
    {
        let fn_ptr = fn_ptr as FnPointer;
        unsafe { self.add_binding(name, Binding::Function {
            ret_type: R::to_bfp_value_type(),
            params: [
                P1::to_bfp_value_type().into(),
            ].into(),
            fn_spec: Box::new(move |_| Ok(FnSpecChoice::Call {fn_ptr, args: [
                FnSpecCallArg::MappedArgument { param_idx: 0 },
            ].into() })),
        }) }
    }

    pub fn add_function_1_map<R, P1, M1>(&mut self, name: String, fn_ptr: fn(P1) -> R, p1: M1) -> Result<(), CommonError>
    where
        R: ToBFPValueType,
        P1: ToBFPValueType,
        M1: Into<FnSpecCallArg>,
    {
        let fn_ptr = fn_ptr as FnPointer;
        let mut params_builder = BindingFunctionParamBuilder::new();

        let p1 = p1.into();
        params_builder.maybe_add_spec_call_arg(&p1, P1::to_bfp_value_type().into())?;

        let params = params_builder.finish()?;
        p1.guard::<P1>(&params)?;

        unsafe { self.add_binding(name, Binding::Function {
            ret_type: R::to_bfp_value_type(),
            params,
            fn_spec: Box::new(move |_| Ok(FnSpecChoice::Call {fn_ptr, args: [
                p1,
            ].into() })),
        }) }
    }

    pub fn add_function_2<R, P1, P2>(&mut self, name: String, fn_ptr: fn(P1, P2) -> R) -> Result<(), CommonError>
    where
        R: ToBFPValueType,
        P1: ToBFPValueType,
        P2: ToBFPValueType,
    {
        let fn_ptr = fn_ptr as FnPointer;
        unsafe { self.add_binding(name, Binding::Function {
            ret_type: R::to_bfp_value_type(),
            params: [
                P1::to_bfp_value_type().into(),
                P2::to_bfp_value_type().into(),
            ].into(),
            fn_spec: Box::new(move |_| Ok(FnSpecChoice::Call {fn_ptr, args: [
                FnSpecCallArg::MappedArgument { param_idx: 0 },
                FnSpecCallArg::MappedArgument { param_idx: 1 },
            ].into() })),
        }) }
    }

    pub fn add_function_2_map<R, P1, M1, P2, M2>(&mut self, name: String, fn_ptr: fn(P1, P2) -> R, p1: M1, p2: M2) -> Result<(), CommonError>
    where
        R: ToBFPValueType,
        P1: ToBFPValueType,
        M1: Into<FnSpecCallArg>,
        P2: ToBFPValueType,
        M2: Into<FnSpecCallArg>,
    {
        let fn_ptr = fn_ptr as FnPointer;
        let mut params_builder = BindingFunctionParamBuilder::new();

        let p1 = p1.into();
        params_builder.maybe_add_spec_call_arg(&p1, P1::to_bfp_value_type().into())?;
        let p2 = p2.into();
        params_builder.maybe_add_spec_call_arg(&p2, P2::to_bfp_value_type().into())?;

        let params = params_builder.finish()?;
        p1.guard::<P1>(&params)?;
        p2.guard::<P2>(&params)?;

        unsafe { self.add_binding(name, Binding::Function {
            ret_type: R::to_bfp_value_type(),
            params,
            fn_spec: Box::new(move |_| Ok(FnSpecChoice::Call {fn_ptr, args: [
                p1,
                p2,
            ].into() })),
        }) }
    }

    pub fn add_function_3<R, P1, P2, P3>(&mut self, name: String, fn_ptr: fn(P1, P2, P3) -> R) -> Result<(), CommonError>
    where
        R: ToBFPValueType,
        P1: ToBFPValueType,
        P2: ToBFPValueType,
        P3: ToBFPValueType,
    {
        let fn_ptr = fn_ptr as FnPointer;
        unsafe { self.add_binding(name, Binding::Function {
            ret_type: R::to_bfp_value_type(),
            params: [
                P1::to_bfp_value_type().into(),
                P2::to_bfp_value_type().into(),
                P3::to_bfp_value_type().into(),
            ].into(),
            fn_spec: Box::new(move |_| Ok(FnSpecChoice::Call {fn_ptr, args: [
                FnSpecCallArg::MappedArgument { param_idx: 0 },
                FnSpecCallArg::MappedArgument { param_idx: 1 },
                FnSpecCallArg::MappedArgument { param_idx: 2 },
            ].into() })),
        }) }
    }

    pub fn add_function_3_map<R, P1, M1, P2, M2, P3, M3>(&mut self, name: String, fn_ptr: fn(P1, P2, P3) -> R, p1: M1, p2: M2, p3: M3) -> Result<(), CommonError>
    where
        R: ToBFPValueType,
        P1: ToBFPValueType,
        M1: Into<FnSpecCallArg>,
        P2: ToBFPValueType,
        M2: Into<FnSpecCallArg>,
        P3: ToBFPValueType,
        M3: Into<FnSpecCallArg>,
    {
        let fn_ptr = fn_ptr as FnPointer;
        let mut params_builder = BindingFunctionParamBuilder::new();

        let p1 = p1.into();
        params_builder.maybe_add_spec_call_arg(&p1, P1::to_bfp_value_type().into())?;
        let p2 = p2.into();
        params_builder.maybe_add_spec_call_arg(&p2, P2::to_bfp_value_type().into())?;
        let p3 = p3.into();
        params_builder.maybe_add_spec_call_arg(&p3, P3::to_bfp_value_type().into())?;

        let params = params_builder.finish()?;
        p1.guard::<P1>(&params)?;
        p2.guard::<P2>(&params)?;
        p3.guard::<P3>(&params)?;

        unsafe { self.add_binding(name, Binding::Function {
            ret_type: R::to_bfp_value_type(),
            params,
            fn_spec: Box::new(move |_| Ok(FnSpecChoice::Call {fn_ptr, args: [
                p1,
                p2,
                p3,
            ].into() })),
        }) }
    }

    pub fn add_function_4<R, P1, P2, P3, P4>(&mut self, name: String, fn_ptr: fn(P1, P2, P3, P4) -> R) -> Result<(), CommonError>
    where
        R: ToBFPValueType,
        P1: ToBFPValueType,
        P2: ToBFPValueType,
        P3: ToBFPValueType,
        P4: ToBFPValueType,
    {
        let fn_ptr = fn_ptr as FnPointer;
        unsafe { self.add_binding(name, Binding::Function {
            ret_type: R::to_bfp_value_type(),
            params: [
                P1::to_bfp_value_type().into(),
                P2::to_bfp_value_type().into(),
                P3::to_bfp_value_type().into(),
                P4::to_bfp_value_type().into(),
            ].into(),
            fn_spec: Box::new(move |_| Ok(FnSpecChoice::Call {fn_ptr, args: [
                FnSpecCallArg::MappedArgument { param_idx: 0 },
                FnSpecCallArg::MappedArgument { param_idx: 1 },
                FnSpecCallArg::MappedArgument { param_idx: 2 },
                FnSpecCallArg::MappedArgument { param_idx: 3 },
            ].into() })),
        }) }
    }

    pub fn add_function_4_map<R, P1, M1, P2, M2, P3, M3, P4, M4>(&mut self, name: String, fn_ptr: fn(P1, P2, P3, P4) -> R, p1: M1, p2: M2, p3: M3, p4: M4) -> Result<(), CommonError>
    where
        R: ToBFPValueType,
        P1: ToBFPValueType,
        M1: Into<FnSpecCallArg>,
        P2: ToBFPValueType,
        M2: Into<FnSpecCallArg>,
        P3: ToBFPValueType,
        M3: Into<FnSpecCallArg>,
        P4: ToBFPValueType,
        M4: Into<FnSpecCallArg>,
    {
        let fn_ptr = fn_ptr as FnPointer;
        let mut params_builder = BindingFunctionParamBuilder::new();

        let p1 = p1.into();
        params_builder.maybe_add_spec_call_arg(&p1, P1::to_bfp_value_type().into())?;
        let p2 = p2.into();
        params_builder.maybe_add_spec_call_arg(&p2, P2::to_bfp_value_type().into())?;
        let p3 = p3.into();
        params_builder.maybe_add_spec_call_arg(&p3, P3::to_bfp_value_type().into())?;
        let p4 = p4.into();
        params_builder.maybe_add_spec_call_arg(&p4, P4::to_bfp_value_type().into())?;

        let params = params_builder.finish()?;
        p1.guard::<P1>(&params)?;
        p2.guard::<P2>(&params)?;
        p3.guard::<P3>(&params)?;
        p4.guard::<P4>(&params)?;

        unsafe { self.add_binding(name, Binding::Function {
            ret_type: R::to_bfp_value_type(),
            params,
            fn_spec: Box::new(move |_| Ok(FnSpecChoice::Call {fn_ptr, args: [
                p1,
                p2,
                p3,
                p4,
            ].into() })),
        }) }
    }

    pub fn add_function_5<R, P1, P2, P3, P4, P5>(&mut self, name: String, fn_ptr: fn(P1, P2, P3, P4, P5) -> R) -> Result<(), CommonError>
    where
        R: ToBFPValueType,
        P1: ToBFPValueType,
        P2: ToBFPValueType,
        P3: ToBFPValueType,
        P4: ToBFPValueType,
        P5: ToBFPValueType,
    {
        let fn_ptr = fn_ptr as FnPointer;
        unsafe { self.add_binding(name, Binding::Function {
            ret_type: R::to_bfp_value_type(),
            params: [
                P1::to_bfp_value_type().into(),
                P2::to_bfp_value_type().into(),
                P3::to_bfp_value_type().into(),
                P4::to_bfp_value_type().into(),
                P5::to_bfp_value_type().into(),
            ].into(),
            fn_spec: Box::new(move |_| Ok(FnSpecChoice::Call {fn_ptr, args: [
                FnSpecCallArg::MappedArgument { param_idx: 0 },
                FnSpecCallArg::MappedArgument { param_idx: 1 },
                FnSpecCallArg::MappedArgument { param_idx: 2 },
                FnSpecCallArg::MappedArgument { param_idx: 3 },
                FnSpecCallArg::MappedArgument { param_idx: 4 },
            ].into() })),
        }) }
    }

    pub fn add_function_5_map<R, P1, M1, P2, M2, P3, M3, P4, M4, P5, M5>(&mut self, name: String, fn_ptr: fn(P1, P2, P3, P4, P5) -> R, p1: M1, p2: M2, p3: M3, p4: M4, p5: M5) -> Result<(), CommonError>
    where
        R: ToBFPValueType,
        P1: ToBFPValueType,
        M1: Into<FnSpecCallArg>,
        P2: ToBFPValueType,
        M2: Into<FnSpecCallArg>,
        P3: ToBFPValueType,
        M3: Into<FnSpecCallArg>,
        P4: ToBFPValueType,
        M4: Into<FnSpecCallArg>,
        P5: ToBFPValueType,
        M5: Into<FnSpecCallArg>,
    {
        let fn_ptr = fn_ptr as FnPointer;
        let mut params_builder = BindingFunctionParamBuilder::new();

        let p1 = p1.into();
        params_builder.maybe_add_spec_call_arg(&p1, P1::to_bfp_value_type().into())?;
        let p2 = p2.into();
        params_builder.maybe_add_spec_call_arg(&p2, P2::to_bfp_value_type().into())?;
        let p3 = p3.into();
        params_builder.maybe_add_spec_call_arg(&p3, P3::to_bfp_value_type().into())?;
        let p4 = p4.into();
        params_builder.maybe_add_spec_call_arg(&p4, P4::to_bfp_value_type().into())?;
        let p5 = p5.into();
        params_builder.maybe_add_spec_call_arg(&p5, P5::to_bfp_value_type().into())?;

        let params = params_builder.finish()?;
        p1.guard::<P1>(&params)?;
        p2.guard::<P2>(&params)?;
        p3.guard::<P3>(&params)?;
        p4.guard::<P4>(&params)?;
        p5.guard::<P5>(&params)?;

        unsafe { self.add_binding(name, Binding::Function {
            ret_type: R::to_bfp_value_type(),
            params,
            fn_spec: Box::new(move |_| Ok(FnSpecChoice::Call {fn_ptr, args: [
                p1,
                p2,
                p3,
                p4,
                p5,
            ].into() })),
        }) }
    }
}