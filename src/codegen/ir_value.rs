use std::error::Error;

use inkwell::{AddressSpace, FloatPredicate, IntPredicate, builder::BuilderError, values::{BasicMetadataValueEnum, BasicValue, BasicValueEnum, FloatValue, IntValue, ValueKind}};

use crate::{analysis::{error::AnalysisError, packed_analysis_node::{FunctionArgument, PackedAnalysisNodeData}, packed_analysis_tree::PackedAnalysisTree}, ast::ast_node::{BinaryOperator, UnaryOperator}, codegen::utils::get_fn_llvm_type, common::{binding::BindingFunctionSpecializationHints, ir_const::IRConst, slab::SlabBindingInfo, value::Value, value_type::ValueType}};

use super::{codegen_context::CodegenContext, error::CodegenError, ir_value_type::IRValueType, utils::get_usize_llvm_type};

pub enum IRValue<'ctx> {
    Int { inner: IntValue<'ctx>, is_signed: bool },
    Float { inner: FloatValue<'ctx> },
}

impl<'ctx> IRValue<'ctx> {
    pub fn from_ast_typed_value<'build>(typed_value: &Value, context: &CodegenContext<'ctx, 'build>) -> Self {
        match typed_value {
            Value::U8 { inner } => Self::Int { inner: context.llvm_context.i8_type().const_int(*inner as u64, false), is_signed: false },
            Value::U16 { inner } => Self::Int { inner: context.llvm_context.i16_type().const_int(*inner as u64, false), is_signed: false },
            Value::U32 { inner } => Self::Int { inner: context.llvm_context.i32_type().const_int(*inner as u64, false), is_signed: false },
            Value::U64 { inner } => Self::Int { inner: context.llvm_context.i64_type().const_int(*inner, false), is_signed: false },
            Value::USize { inner } => Self::Int { inner: get_usize_llvm_type(context.llvm_context).const_int(*inner as u64, false), is_signed: false },
            Value::I8 { inner } => Self::Int { inner: context.llvm_context.i8_type().const_int(*inner as u64, true), is_signed: true },
            Value::I16 { inner } => Self::Int { inner: context.llvm_context.i16_type().const_int(*inner as u64, true), is_signed: true },
            Value::I32 { inner } => Self::Int { inner: context.llvm_context.i32_type().const_int(*inner as u64, true), is_signed: true },
            Value::I64 { inner } => Self::Int { inner: context.llvm_context.i64_type().const_int(*inner as u64, true), is_signed: true },
            Value::F32 { inner } => Self::Float { inner: context.llvm_context.f32_type().const_float(*inner as f64) },
            Value::F64 { inner } => Self::Float { inner: context.llvm_context.f64_type().const_float(*inner) },
            Value::Bool { inner } => Self::Int { inner: context.llvm_context.bool_type().const_int(*inner as u64, false), is_signed: false },
        }
    }

    pub fn cast_if_needed<'build>(self, from: ValueType, to: ValueType, context: &CodegenContext<'ctx, 'build>) -> Result<Self, Box<dyn Error>> {
        if from == to { return Ok(self) }

        let builder = context.builder;

        if to == ValueType::Bool {
            // XXX llvm casts to bool in a weird way, so we need a special case
            //     to convert it correctly
            Ok(match self {
                IRValue::Int { inner, is_signed: _ } => {
                    IRValue::Int {
                        inner: context.builder.build_int_compare(IntPredicate::NE, inner, inner.get_type().const_zero(), "")?,
                        is_signed: false,
                    }
                },
                IRValue::Float { inner } => {
                    IRValue::Int {
                        // ONE, not UNE; NaN is falsy
                        inner: context.builder.build_float_compare(FloatPredicate::ONE, inner, inner.get_type().const_zero(), "")?,
                        is_signed: false,
                    }
                },
            })
        } else {
            Ok(match IRValueType::from_value_type(&to, context.llvm_context) {
                IRValueType::Int { llvm: wanted_llvm, is_signed: to_signed } => {
                    match self {
                        Self::Int { inner, is_signed: _ } => {
                            let val = builder.build_int_cast_sign_flag(inner, wanted_llvm, to_signed, "")?;
                            IRValue::Int { inner: val, is_signed: to_signed }
                        },
                        Self::Float { inner } => {
                            let val = if to_signed {
                                builder.build_float_to_signed_int(inner, wanted_llvm, "")?
                            } else {
                                builder.build_float_to_unsigned_int(inner, wanted_llvm, "")?
                            };
                            IRValue::Int { inner: val, is_signed: to_signed }
                        },
                    }
                },
                IRValueType::Float { llvm: wanted_llvm } => {
                    match self {
                        Self::Int { inner, is_signed: from_signed } => {
                            let val = if from_signed {
                                builder.build_signed_int_to_float(inner, wanted_llvm, "")?
                            } else {
                                builder.build_unsigned_int_to_float(inner, wanted_llvm, "")?
                            };
                            IRValue::Float { inner: val }
                        },
                        Self::Float { inner } => {
                            let val = builder.build_float_cast(inner, wanted_llvm, "")?;
                            IRValue::Float { inner: val }
                        },
                    }
                },
            })
        }
    }

    pub fn ref_inner_generic(&'ctx self) -> &'ctx dyn BasicValue<'ctx> {
        match self {
            Self::Int { inner, is_signed: _ } => inner,
            Self::Float { inner } => inner,
        }
    }

    pub fn to_meta_value(&self) -> BasicMetadataValueEnum<'ctx> {
        match self {
            Self::Int { inner, is_signed: _ } => (*inner).into(),
            Self::Float { inner } => (*inner).into(),
        }
    }

    pub fn from_aast<'build>(aast: &PackedAnalysisTree, context: &CodegenContext<'ctx, 'build>) -> Result<Self, Box<dyn Error>> {
        Self::from_aast_node(aast, aast.nodes.len() - 1, context)
    }

    fn from_binary_op<'build, BI, BF>(aast: &PackedAnalysisTree, resolved_type: ValueType, left_idx: usize, right_idx: usize, context: &CodegenContext<'ctx, 'build>, build_int: BI, build_float: BF) -> Result<IRValue<'ctx>, Box<dyn Error>>
    where
        BI: FnOnce(IntValue<'ctx>, IntValue<'ctx>, bool, &CodegenContext<'ctx, 'build>) -> Result<IntValue<'ctx>, BuilderError>,
        BF: FnOnce(FloatValue<'ctx>, FloatValue<'ctx>, &CodegenContext<'ctx, 'build>) -> Result<FloatValue<'ctx>, BuilderError>,
    {
        let left_val = Self::from_aast_node(aast, left_idx, context)?;
        let right_val = Self::from_aast_node(aast, right_idx, context)?;

        let right_val = right_val.cast_if_needed(aast.get_node_type(right_idx)?, resolved_type, context)?;
        Ok(match left_val.cast_if_needed(aast.get_node_type(left_idx)?, resolved_type, context)? {
            IRValue::Int { inner: left_inner, is_signed } => IRValue::Int {
                inner: build_int(left_inner, right_val.try_into()?, is_signed, context)?,
                is_signed,
            },
            IRValue::Float { inner: left_inner } => IRValue::Float {
                inner: build_float(left_inner, right_val.try_into()?, context)?,
            },
        })
    }

    fn from_compare_op<'build>(aast: &PackedAnalysisTree, left_idx: usize, right_idx: usize, context: &CodegenContext<'ctx, 'build>, uint_pred: IntPredicate, sint_pred: IntPredicate, float_pred: FloatPredicate) -> Result<IRValue<'ctx>, Box<dyn Error>> {
        let left_val = Self::from_aast_node(aast, left_idx, context)?;
        let left_type = aast.get_node_type(left_idx)?;
        let right_val = Self::from_aast_node(aast, right_idx, context)?;
        let right_type = aast.get_node_type(right_idx)?;
        let resolved_type = ValueType::widen(left_type, right_type)?;

        let right_val = right_val.cast_if_needed(right_type, resolved_type, context)?;
        Ok(match left_val.cast_if_needed(left_type, resolved_type, context)? {
            IRValue::Int { inner: left_inner, is_signed } => IRValue::Int {
                inner: context.builder.build_int_compare(if is_signed { sint_pred } else { uint_pred }, left_inner, right_val.try_into()?, "")?,
                is_signed: false,
            },
            IRValue::Float { inner: left_inner } => IRValue::Int {
                inner: context.builder.build_float_compare(float_pred, left_inner, right_val.try_into()?, "")?,
                is_signed: false,
            },
        })
    }

    fn from_branching_expr<'build, CC, LC, RC>(aast: &PackedAnalysisTree, context: &CodegenContext<'ctx, 'build>, out_type: ValueType, cond_callback: CC, left_callback: LC, right_callback: RC) -> Result<Self, Box<dyn Error>>
    where
        CC: FnOnce(&PackedAnalysisTree, &CodegenContext<'ctx, 'build>) -> Result<(Self, ValueType), Box<dyn Error>>,
        LC: FnOnce(&PackedAnalysisTree, &CodegenContext<'ctx, 'build>) -> Result<(Self, ValueType), Box<dyn Error>>,
        RC: FnOnce(&PackedAnalysisTree, &CodegenContext<'ctx, 'build>) -> Result<(Self, ValueType), Box<dyn Error>>,
    {
        let (cond_val, cond_type) = cond_callback(aast, context)?;
        let cond_bool: IntValue<'ctx> = cond_val.cast_if_needed(cond_type, ValueType::Bool, context)?.try_into()?;

        let then_block = context.llvm_context.append_basic_block(*context.func, "");
        let else_block = context.llvm_context.append_basic_block(*context.func, "");
        let after_block = context.llvm_context.append_basic_block(*context.func, "");
        context.builder.build_conditional_branch(cond_bool, then_block, else_block)?;

        context.builder.position_at_end(then_block);
        let (left_val, left_type) = left_callback(aast, context)?;
        let left_val = left_val.cast_if_needed(left_type, out_type, context)?;
        context.builder.build_unconditional_branch(after_block)?;

        context.builder.position_at_end(else_block);
        let (right_val, right_type) = right_callback(aast, context)?;
        let right_val = right_val.cast_if_needed(right_type, out_type, context)?;
        context.builder.build_unconditional_branch(after_block)?;

        context.builder.position_at_end(after_block);

        Ok(match left_val {
            IRValue::Int { inner: left_inner, is_signed } => {
                let phi = context.builder.build_phi(left_inner.get_type(), "")?;
                phi.add_incoming(&[(&left_inner, then_block), (&TryInto::<IntValue<'ctx>>::try_into(right_val)?, else_block)]);
                IRValue::Int { inner: phi.as_basic_value().into_int_value(), is_signed }
            },
            IRValue::Float { inner: left_inner } => {
                let phi = context.builder.build_phi(left_inner.get_type(), "")?;
                phi.add_incoming(&[(&left_inner, then_block), (&TryInto::<FloatValue<'ctx>>::try_into(right_val)?, else_block)]);
                IRValue::Float { inner: phi.as_basic_value().into_float_value() }
            },
        })
    }

    fn from_aast_node<'build>(aast: &PackedAnalysisTree, idx: usize, context: &CodegenContext<'ctx, 'build>) -> Result<Self, Box<dyn Error>> {
        let resolved_type = aast.get_node_type(idx)?;

        Ok(match &aast.nodes[idx].data {
            PackedAnalysisNodeData::TypedValue { value } => Self::from_ast_typed_value(value, context),
            PackedAnalysisNodeData::UntypedValue { .. } => return Err(Box::new(AnalysisError::BadAnalysis)),
            PackedAnalysisNodeData::FunctionCall { args, fn_spec } => {
                let mut arg_types = Vec::<ValueType>::new();
                let mut llvm_args = Vec::<BasicMetadataValueEnum<'ctx>>::new();
                let mut spec_hint_consts = Vec::<Option<IRConst>>::new();

                for arg in args {
                    match arg {
                        FunctionArgument::Parameter { idx: arg_idx, expected_type: arg_type } => {
                            let arg_idx = *arg_idx;
                            let arg_type = *arg_type;
                            arg_types.push(arg_type);
                            let llvm_val = Self::from_aast_node(aast, arg_idx, context)?.cast_if_needed(aast.get_node_type(arg_idx)?, arg_type, context)?;
                            spec_hint_consts.push(llvm_val.get_ir_const());
                            llvm_args.push(llvm_val.to_meta_value());
                        },
                        FunctionArgument::ConstArgument { value } => {
                            arg_types.push(value.get_value_type());
                            llvm_args.push(Self::from_ast_typed_value(value, context).to_meta_value());
                        },
                        FunctionArgument::HiddenStateArgument { hidden_state_idx, slab_value_type, cast_to_type } => {
                            let hidden_state_idx = *hidden_state_idx;
                            if hidden_state_idx >= context.slab.get_hidden_state_count() {
                                return Err(Box::new(CodegenError::UnknownHiddenState { idx: hidden_state_idx }));
                            }

                            let mut ir_slab_value = IRValue::from_slab_value(hidden_state_idx, slab_value_type, context)?;
                            if let Some(cast_to_type) = cast_to_type {
                                ir_slab_value = ir_slab_value.cast_if_needed(*slab_value_type, *cast_to_type, context)?;
                                arg_types.push(*cast_to_type);
                            } else {
                                arg_types.push(*slab_value_type);
                            }

                            llvm_args.push(ir_slab_value.to_meta_value());
                        },
                    }
                }

                let fn_ptr = fn_spec(BindingFunctionSpecializationHints { consts: spec_hint_consts.into() });
                let fn_type = get_fn_llvm_type(context.llvm_context, resolved_type, arg_types);
                let ptr_type = context.llvm_context.ptr_type(AddressSpace::default());
                let ptr_val = get_usize_llvm_type(context.llvm_context).const_int(fn_ptr.addr() as u64, false).const_to_pointer(ptr_type);
                let ret_val = context.builder.build_indirect_call(fn_type, ptr_val, llvm_args.as_slice(), "")?;

                match ret_val.try_as_basic_value() {
                    ValueKind::Basic(basic_value_enum) => {
                        match basic_value_enum {
                            BasicValueEnum::IntValue(inner) => IRValue::from_int_value(inner, resolved_type)?,
                            BasicValueEnum::FloatValue(inner) => IRValue::from_float_value(inner, resolved_type)?,
                            _ => return Err(Box::new(CodegenError::UnexpectedBasicValueEnum)),
                        }
                    },
                    ValueKind::Instruction(..) => return Err(Box::new(CodegenError::UnexpectedFunctionReturnValue)),
                }
            },
            PackedAnalysisNodeData::UnaryOperation { operator, right_idx } => {
                let inner_val = Self::from_aast_node(aast, *right_idx, context)?;

                match operator {
                    UnaryOperator::Negate => {
                        match inner_val.cast_if_needed(aast.get_node_type(*right_idx)?, resolved_type, context)? {
                            IRValue::Int { inner, is_signed } => IRValue::Int {
                                inner: context.builder.build_int_neg(inner, "")?,
                                is_signed,
                            },
                            IRValue::Float { inner } => IRValue::Float {
                                inner: context.builder.build_float_neg(inner, "")?,
                            },
                        }
                    },
                    UnaryOperator::LogicalNot => {
                        // cast inner to bool, and then compare inner with 0:
                        // 0 == 0 -> 1
                        // 1 == 0 -> 0
                        match inner_val {
                            IRValue::Int { inner, is_signed: _ } => IRValue::Int {
                                inner: context.builder.build_int_compare(IntPredicate::EQ, inner, inner.get_type().const_zero(), "")?,
                                is_signed: false,
                            },
                            IRValue::Float { inner } => IRValue::Int {
                                // UEQ, not OEQ; NaN is falsy
                                inner: context.builder.build_float_compare(FloatPredicate::UEQ, inner, inner.get_type().const_zero(), "")?,
                                is_signed: false,
                            },
                        }
                    },
                }
            },
            PackedAnalysisNodeData::BinaryOperation { operator, left_idx, right_idx } => {
                match operator {
                    BinaryOperator::Mul => Self::from_binary_op(aast, resolved_type, *left_idx, *right_idx, context, |lhs, rhs, _, context|{
                        context.builder.build_int_mul(lhs, rhs, "")
                    }, |lhs, rhs, context|{
                        context.builder.build_float_mul(lhs, rhs, "")
                    })?,
                    BinaryOperator::Div => Self::from_binary_op(aast, resolved_type, *left_idx, *right_idx, context, |lhs, rhs, is_signed, context|{
                        if is_signed {
                            context.builder.build_int_signed_div(lhs, rhs, "")
                        } else {
                            context.builder.build_int_unsigned_div(lhs, rhs, "")
                        }
                    }, |lhs, rhs, context|{
                        context.builder.build_float_div(lhs, rhs, "")
                    })?,
                    // FIXME: verify this behaviour. it feels off, as this isn't modulo: https://llvm.org/docs/LangRef.html#urem-instruction
                    BinaryOperator::Mod => Self::from_binary_op(aast, resolved_type, *left_idx, *right_idx, context, |lhs, rhs, is_signed, context|{
                        if is_signed {
                            context.builder.build_int_signed_rem(lhs, rhs, "")
                        } else {
                            context.builder.build_int_unsigned_rem(lhs, rhs, "")
                        }
                    }, |lhs, rhs, context|{
                        context.builder.build_float_rem(lhs, rhs, "")
                    })?,
                    BinaryOperator::Add => Self::from_binary_op(aast, resolved_type, *left_idx, *right_idx, context, |lhs, rhs, _, context|{
                        context.builder.build_int_add(lhs, rhs, "")
                    }, |lhs, rhs, context|{
                        context.builder.build_float_add(lhs, rhs, "")
                    })?,
                    BinaryOperator::Sub => Self::from_binary_op(aast, resolved_type, *left_idx, *right_idx, context, |lhs, rhs, _, context|{
                        context.builder.build_int_sub(lhs, rhs, "")
                    }, |lhs, rhs, context|{
                        context.builder.build_float_sub(lhs, rhs, "")
                    })?,
                    BinaryOperator::Equals => Self::from_compare_op(aast, *left_idx, *right_idx, context, IntPredicate::EQ, IntPredicate::EQ, FloatPredicate::OEQ)?,
                    BinaryOperator::NotEquals => Self::from_compare_op(aast, *left_idx, *right_idx, context, IntPredicate::NE, IntPredicate::NE, FloatPredicate::ONE)?,
                    BinaryOperator::LesserThanEquals => Self::from_compare_op(aast, *left_idx, *right_idx, context, IntPredicate::ULE, IntPredicate::SLE, FloatPredicate::OLE)?,
                    BinaryOperator::GreaterThanEquals => Self::from_compare_op(aast, *left_idx, *right_idx, context, IntPredicate::UGE, IntPredicate::SGE, FloatPredicate::OGE)?,
                    BinaryOperator::LesserThan => Self::from_compare_op(aast, *left_idx, *right_idx, context, IntPredicate::ULT, IntPredicate::SLT, FloatPredicate::OLT)?,
                    BinaryOperator::GreaterThan => Self::from_compare_op(aast, *left_idx, *right_idx, context, IntPredicate::UGT, IntPredicate::SGT, FloatPredicate::OGT)?,
                    BinaryOperator::LogicalAnd => {
                        let left_idx = *left_idx;
                        let right_idx = *right_idx;

                        Self::from_branching_expr(aast, context, ValueType::Bool, |aast, context| {
                            Ok((Self::from_aast_node(aast, left_idx, context)?, aast.get_node_type(left_idx)?))
                        }, |aast, context| {
                            Ok((Self::from_aast_node(aast, right_idx, context)?, aast.get_node_type(right_idx)?))
                        }, |_, context| {
                            Ok((Self::from_ast_typed_value(&Value::Bool { inner: false }, context), ValueType::Bool))
                        })?
                    },
                    BinaryOperator::LogicalOr => {
                        let left_idx = *left_idx;
                        let right_idx = *right_idx;

                        Self::from_branching_expr(aast, context, ValueType::Bool, |aast, context| {
                            Ok((Self::from_aast_node(aast, left_idx, context)?, aast.get_node_type(left_idx)?))
                        }, |_, context| {
                            Ok((Self::from_ast_typed_value(&Value::Bool { inner: true }, context), ValueType::Bool))
                        }, |aast, context| {
                            Ok((Self::from_aast_node(aast, right_idx, context)?, aast.get_node_type(right_idx)?))
                        })?
                    },
                }
            },
            PackedAnalysisNodeData::Variable { name } => {
                let info = match context.slab.get_binding_info(name) {
                    Some(x) => Ok(x),
                    None => Err(CodegenError::UnknownBinding { name: name.clone() }),
                }?;

                let slab_idx = *match info {
                    SlabBindingInfo::Variable { idx, value_type } => {
                        if *value_type != resolved_type {
                            Err(CodegenError::BadBindingType { name: name.clone(), actual_type: resolved_type, expected_type: *value_type })
                        } else {
                            Ok(idx)
                        }
                    },
                    SlabBindingInfo::Function { .. } => {
                        Err(CodegenError::BadBindingKind { name: name.clone(), is_var: false })
                    },
                }?;

                IRValue::from_slab_value(slab_idx, &resolved_type, context)?
            },
            PackedAnalysisNodeData::Ternary { cond_idx, left_idx, right_idx } => {
                let cond_idx = *cond_idx;
                let left_idx = *left_idx;
                let right_idx = *right_idx;

                Self::from_branching_expr(aast, context, resolved_type, |aast, context| {
                    Ok((Self::from_aast_node(aast, cond_idx, context)?, aast.get_node_type(cond_idx)?))
                }, |aast, context| {
                    Ok((Self::from_aast_node(aast, left_idx, context)?, aast.get_node_type(left_idx)?))
                }, |aast, context| {
                    Ok((Self::from_aast_node(aast, right_idx, context)?, aast.get_node_type(right_idx)?))
                })?
            },
        })
    }

    fn from_int_value(inner: IntValue<'ctx>, value_type: ValueType) -> Result<Self, CodegenError> {
        match value_type {
            ValueType::U8 |
            ValueType::U16 |
            ValueType::U32 |
            ValueType::U64 |
            ValueType::USize |
            ValueType::Bool => Ok(IRValue::Int { inner, is_signed: false }),
            ValueType::I8 |
            ValueType::I16 |
            ValueType::I32 |
            ValueType::I64 => Ok(IRValue::Int { inner, is_signed: true }),
            _ => Err(CodegenError::UnexpectedBaseType),
        }
    }

    fn from_float_value(inner: FloatValue<'ctx>, value_type: ValueType) -> Result<Self, CodegenError> {
        match value_type {
            ValueType::F32 |
            ValueType::F64 => Ok(IRValue::Float { inner }),
            _ => Err(CodegenError::UnexpectedBaseType),
        }
    }

    fn from_slab_value<'build>(slab_idx: usize, slab_value_type: &ValueType, context: &CodegenContext<'ctx, 'build>) -> Result<Self, Box<dyn Error>> {
        let pointee_type = IRValueType::from_value_type(slab_value_type, context.llvm_context);
        let ptr = context.slab.get_address(slab_idx);
        let ptr_type = context.llvm_context.ptr_type(AddressSpace::default());
        let ptr_val = get_usize_llvm_type(context.llvm_context).const_int(ptr as u64, false).const_to_pointer(ptr_type);

        let res = match pointee_type {
            IRValueType::Int { llvm, is_signed: _ } => context.builder.build_load(llvm, ptr_val, ""),
            IRValueType::Float { llvm } => context.builder.build_load(llvm, ptr_val, ""),
        }?;

        match res {
            BasicValueEnum::IntValue(inner) => Ok(IRValue::from_int_value(inner, *slab_value_type)?),
            BasicValueEnum::FloatValue(inner) => Ok(IRValue::from_float_value(inner, *slab_value_type)?),
            _ => Err(Box::new(CodegenError::UnexpectedBasicValueEnum)),
        }
    }

    fn get_ir_const(&self) -> Option<IRConst> {
        match *self {
            IRValue::Int { inner, is_signed } => {
                if is_signed {
                    match inner.get_sign_extended_constant() {
                        Some(inner) => Some(IRConst::Int { inner }),
                        None => None,
                    }
                } else {
                    match inner.get_zero_extended_constant() {
                        Some(inner) => Some(IRConst::Uint { inner }),
                        None => None,
                    }
                }
            },
            IRValue::Float { inner } => {
                match inner.get_constant() {
                    Some((inner, _)) => Some(IRConst::Float { inner }),
                    None => None,
                }
            },
        }
    }

    /*fn get_const_value(&self, value_type: &ValueType) -> Result<Option<Value>, CodegenError> {
        let ir_const = match self.get_ir_const() {
            Some(x) => x,
            None => return Ok(None),
        };

        Ok(Some(match ir_const {
            IRConst::Int { inner } => {
                match value_type {
                    ValueType::I8 => Value::I8 { inner: inner.try_into().map_err(|_| CodegenError::UnexpectedBaseType)? },
                    ValueType::I16 => Value::I16 { inner: inner.try_into().map_err(|_| CodegenError::UnexpectedBaseType)? },
                    ValueType::I32 => Value::I32 { inner: inner.try_into().map_err(|_| CodegenError::UnexpectedBaseType)? },
                    ValueType::I64 => Value::I64 { inner },
                    _ => return Err(CodegenError::UnexpectedBaseType),
                }
            },
            IRConst::Uint { inner } => {
                match value_type {
                    ValueType::U8 => Value::U8 { inner: inner.try_into().map_err(|_| CodegenError::UnexpectedBaseType)? },
                    ValueType::U16 => Value::U16 { inner: inner.try_into().map_err(|_| CodegenError::UnexpectedBaseType)? },
                    ValueType::U32 => Value::U32 { inner: inner.try_into().map_err(|_| CodegenError::UnexpectedBaseType)? },
                    ValueType::U64 => Value::U64 { inner },
                    ValueType::USize => Value::USize { inner: inner.try_into().map_err(|_| CodegenError::UnexpectedBaseType)? },
                    ValueType::Bool => {
                        if inner == 1 {
                            Value::Bool { inner: true }
                        } else if inner == 0 {
                            Value::Bool { inner: false }
                        } else {
                            return Err(CodegenError::UnexpectedBaseType);
                        }
                    },
                    _ => return Err(CodegenError::UnexpectedBaseType),
                }
            },
            IRConst::Float { inner } => {
                match value_type {
                    ValueType::F32 => Value::F32 { inner: inner as f32 },
                    ValueType::F64 => Value::F64 { inner },
                    _ => return Err(CodegenError::UnexpectedBaseType),
                }
            },
        }))
    }*/
}

impl<'ctx> TryFrom<IRValue<'ctx>> for IntValue<'ctx> {
    type Error = CodegenError;

    fn try_from(val: IRValue<'ctx>) -> Result<Self, <Self as TryFrom<IRValue<'ctx>>>::Error> {
        match val {
            IRValue::Int { inner, is_signed: _ } => Ok(inner),
            IRValue::Float { .. } => Err(CodegenError::UnexpectedBaseType),
        }
    }
}

impl<'ctx> TryFrom<IRValue<'ctx>> for FloatValue<'ctx> {
    type Error = CodegenError;

    fn try_from(val: IRValue<'ctx>) -> Result<Self, <Self as TryFrom<IRValue<'ctx>>>::Error> {
        match val {
            IRValue::Int { .. } => Err(CodegenError::UnexpectedBaseType),
            IRValue::Float { inner } => Ok(inner),
        }
    }
}