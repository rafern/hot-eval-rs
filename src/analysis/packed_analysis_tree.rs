use std::error::Error;

use crate::{analysis::packed_analysis_node::{PackedAnalysisFunctionArg, PackedAnalysisNodeData}, ast::ast_node::{BinaryOperator, Expression, UnaryOperator}, common::{binding::Binding, table::Table, untyped_value::UntypedValue, value_type::ValueType}};

use super::{error::AnalysisError, packed_analysis_node::PackedAnalysisNode};

pub struct PackedAnalysisTree<'table> {
    pub nodes: Vec<PackedAnalysisNode<'table>>,
}

impl<'table> PackedAnalysisTree<'table> {
    fn ast_to_analysis_node(&mut self, ast_node: &Expression, table: &'table Table) -> Result<usize, AnalysisError> {
        let (node, this_idx) = match ast_node {
            Expression::TypedValue { value } => {
                (PackedAnalysisNode {
                    resolved_type: Some(value.get_value_type()),
                    data: PackedAnalysisNodeData::TypedValue { value: value.clone() },
                    parent_idx: None,
                }, self.nodes.len())
            },
            Expression::UntypedValue { value } => {
                (PackedAnalysisNode {
                    resolved_type: None,
                    data: PackedAnalysisNodeData::UntypedValue { value: value.clone() },
                    parent_idx: None,
                }, self.nodes.len())
            },
            Expression::FunctionCall { name, arguments } => {
                let binding = table.get_binding(name).ok_or_else(|| { AnalysisError::UnknownBinding { name: name.clone() } })?;
                let actual_argc = arguments.len();
                let (ret_type, params, fn_spec) = match binding {
                    Binding::Const { .. } |
                    Binding::Variable { .. } => {
                        return Err(AnalysisError::BadBindingKind { name: name.clone(), is_var: true })
                    },
                    Binding::Function { ret_type, params, fn_spec } => {
                        let expected_argc = params.len();
                        if expected_argc != actual_argc {
                            return Err(AnalysisError::BadArguments { name: name.clone(), expected_argc, actual_argc })
                        }

                        (ret_type, params, fn_spec)
                    },
                };

                let mut args = Vec::<PackedAnalysisFunctionArg>::new();
                for a in 0..params.len() {
                    args.push(PackedAnalysisFunctionArg {
                        idx: self.ast_to_analysis_node(&arguments[a], table)?,
                        expected_type: params[a],
                    });
                }

                let this_idx = self.nodes.len();

                for arg in &args {
                    self.nodes[arg.idx].parent_idx = Some(this_idx);
                }

                (PackedAnalysisNode {
                    resolved_type: Some(*ret_type),
                    data: PackedAnalysisNodeData::FunctionCall { args: args.into(), fn_spec },
                    parent_idx: None,
                }, this_idx)
            },
            Expression::UnaryOperation { operator, right } => {
                let right_idx = self.ast_to_analysis_node(right, table)?;
                let this_idx = self.nodes.len();
                self.nodes[right_idx].parent_idx = Some(this_idx);

                (PackedAnalysisNode {
                    resolved_type: match operator {
                        UnaryOperator::Negate => None,
                        UnaryOperator::LogicalNot => Some(ValueType::Bool),
                    },
                    data: PackedAnalysisNodeData::UnaryOperation { operator: operator.clone(), right_idx },
                    parent_idx: None,
                }, this_idx)
            },
            Expression::BinaryOperation { operator, left, right } => {
                let left_idx = self.ast_to_analysis_node(left, table)?;
                let right_idx = self.ast_to_analysis_node(right, table)?;
                let this_idx = self.nodes.len();
                self.nodes[left_idx].parent_idx = Some(this_idx);
                self.nodes[right_idx].parent_idx = Some(this_idx);

                (PackedAnalysisNode {
                    resolved_type: match operator {
                        BinaryOperator::Mul |
                        BinaryOperator::Div |
                        BinaryOperator::Mod |
                        BinaryOperator::Add |
                        BinaryOperator::Sub => None,
                        BinaryOperator::Equals |
                        BinaryOperator::NotEquals |
                        BinaryOperator::LesserThanEquals |
                        BinaryOperator::GreaterThanEquals |
                        BinaryOperator::LesserThan |
                        BinaryOperator::GreaterThan |
                        BinaryOperator::LogicalAnd |
                        BinaryOperator::LogicalOr => Some(ValueType::Bool),
                    },
                    data: PackedAnalysisNodeData::BinaryOperation { operator: operator.clone(), left_idx, right_idx },
                    parent_idx: None,
                }, this_idx)
            },
            Expression::Binding { name } => {
                let binding = table.get_binding(name).ok_or_else(|| { AnalysisError::UnknownBinding { name: name.clone() } })?;
                let new_node = match binding {
                    Binding::Const { value } => PackedAnalysisNode {
                        resolved_type: Some(value.get_value_type()),
                        data: PackedAnalysisNodeData::TypedValue { value: value.clone() },
                        parent_idx: None,
                    },
                    Binding::Variable { value_type } => PackedAnalysisNode {
                        resolved_type: Some(value_type.clone()),
                        data: PackedAnalysisNodeData::Variable { name: name.clone() },
                        parent_idx: None,
                    },
                    Binding::Function { .. } => {
                        return Err(AnalysisError::BadBindingKind { name: name.clone(), is_var: false })
                    }
                };

                (new_node, self.nodes.len())
            },
            Expression::Ternary { cond, left, right } => {
                let cond_idx = self.ast_to_analysis_node(cond, table)?;
                let left_idx = self.ast_to_analysis_node(left, table)?;
                let right_idx = self.ast_to_analysis_node(right, table)?;
                let this_idx = self.nodes.len();
                self.nodes[cond_idx].parent_idx = Some(this_idx);
                self.nodes[left_idx].parent_idx = Some(this_idx);
                self.nodes[right_idx].parent_idx = Some(this_idx);

                (PackedAnalysisNode {
                    resolved_type: None,
                    data: PackedAnalysisNodeData::Ternary { cond_idx, left_idx, right_idx },
                    parent_idx: None,
                }, this_idx)
            },
        };

        self.nodes.push(node);
        debug_assert!(self.nodes.len() - 1 == this_idx);
        Ok(this_idx)
    }

    pub fn from_ast(ast_root_node: &Expression, table: &'table Table) -> Result<PackedAnalysisTree<'table>, Box<dyn Error>> {
        let mut tree = PackedAnalysisTree { nodes: Vec::new() };
        tree.ast_to_analysis_node(ast_root_node, table)?;
        tree.semantic_analysis()?;
        Ok(tree)
    }

    fn propagate_type_from_inner(&mut self, idx: usize) -> Result<bool, Box<dyn Error>> {
        if self.nodes[idx].resolved_type.is_some() { return Ok(false) }

        let new_type = match &self.nodes[idx].data {
            PackedAnalysisNodeData::TypedValue { .. } |
            PackedAnalysisNodeData::UntypedValue { .. } |
            PackedAnalysisNodeData::FunctionCall { .. } |
            PackedAnalysisNodeData::Variable { .. } => unreachable!(),
            PackedAnalysisNodeData::UnaryOperation { operator, right_idx } => {
                let right_idx = *right_idx;

                match operator {
                    UnaryOperator::Negate => ValueType::to_signed_optional(self.nodes[right_idx].resolved_type)?,
                    UnaryOperator::LogicalNot => unreachable!(),
                }
            },
            PackedAnalysisNodeData::BinaryOperation { operator, left_idx, right_idx } => {
                let left_idx = *left_idx;
                let right_idx = *right_idx;

                match operator {
                    BinaryOperator::Mul |
                    BinaryOperator::Div |
                    BinaryOperator::Mod |
                    BinaryOperator::Add |
                    BinaryOperator::Sub => {
                        ValueType::widen_optional_non_greedy(self.nodes[left_idx].resolved_type, self.nodes[right_idx].resolved_type)?
                    },
                    _ => unreachable!(),
                }
            },
            PackedAnalysisNodeData::Ternary { cond_idx: _, left_idx, right_idx } => {
                let left_idx = *left_idx;
                let right_idx = *right_idx;

                ValueType::widen_optional_non_greedy(self.nodes[left_idx].resolved_type, self.nodes[right_idx].resolved_type)?
            },
        };

        if let Some(new_type) = new_type {
            self.nodes[idx].resolved_type = Some(new_type);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn resolve_types_from_inner(&mut self) -> Result<bool, Box<dyn Error>> {
        let mut had_changes = false;
        for idx in 0..self.nodes.len() {
            let node = &self.nodes[idx];
            match node.resolved_type { Some(_) => {}, None => continue };
            let parent_idx = match node.parent_idx { Some(x) => x, None => continue };
            had_changes = self.propagate_type_from_inner(parent_idx)? || had_changes;
        }

        Ok(had_changes)
    }

    fn get_child_input_hint(&self, parent_idx: usize, child_idx: usize, parent_hint: Option<ValueType>) -> Result<Option<ValueType>, Box<dyn Error>> {
        let node = &self.nodes[parent_idx];
        Ok(match &node.data {
            PackedAnalysisNodeData::TypedValue { .. } |
            PackedAnalysisNodeData::UntypedValue { .. } |
            PackedAnalysisNodeData::Variable { .. } => return Err(Box::new(AnalysisError::BadAnalysis)),
            PackedAnalysisNodeData::FunctionCall { args, fn_spec: _  } => 'slfc_match: {
                for PackedAnalysisFunctionArg { idx, expected_type } in args {
                    if child_idx == *idx {
                        break 'slfc_match Some(*expected_type);
                    }
                }

                return Err(Box::new(AnalysisError::BadAnalysis))
            },
            PackedAnalysisNodeData::UnaryOperation { operator, right_idx } => {
                if child_idx != *right_idx {
                    return Err(Box::new(AnalysisError::BadAnalysis))
                }

                match operator {
                    UnaryOperator::Negate => {
                        if let Some(parent_hint) = parent_hint {
                            Some(parent_hint.to_signed()?)
                        } else {
                            ValueType::to_signed_optional(node.resolved_type)?
                        }
                    },
                    UnaryOperator::LogicalNot => Some(ValueType::Bool),
                }
            },
            PackedAnalysisNodeData::BinaryOperation { operator, left_idx, right_idx } => {
                let left_idx = *left_idx;
                let right_idx = *right_idx;

                if child_idx != left_idx && child_idx != right_idx {
                    return Err(Box::new(AnalysisError::BadAnalysis))
                }

                match operator {
                    BinaryOperator::Mul |
                    BinaryOperator::Div |
                    BinaryOperator::Mod |
                    BinaryOperator::Add |
                    BinaryOperator::Sub => {
                        if node.resolved_type.is_some() {
                            node.resolved_type
                        } else if parent_hint.is_some() {
                            parent_hint
                        } else {
                            ValueType::widen_optional_greedy(self.nodes[left_idx].resolved_type, self.nodes[right_idx].resolved_type)?
                        }
                    },
                    BinaryOperator::Equals |
                    BinaryOperator::NotEquals |
                    BinaryOperator::LesserThanEquals |
                    BinaryOperator::GreaterThanEquals |
                    BinaryOperator::LesserThan |
                    BinaryOperator::GreaterThan => ValueType::widen_optional_greedy(self.nodes[left_idx].resolved_type, self.nodes[right_idx].resolved_type)?,
                    BinaryOperator::LogicalAnd |
                    BinaryOperator::LogicalOr => Some(ValueType::Bool),
                }
            },
            PackedAnalysisNodeData::Ternary { cond_idx, left_idx, right_idx } => {
                let cond_idx = *cond_idx;
                let left_idx = *left_idx;
                let right_idx = *right_idx;

                if child_idx == cond_idx {
                    Some(ValueType::Bool)
                } else if child_idx != left_idx && child_idx != right_idx {
                    return Err(Box::new(AnalysisError::BadAnalysis))
                } else {
                    ValueType::widen_optional_greedy(self.nodes[left_idx].resolved_type, self.nodes[right_idx].resolved_type)?
                }
            },
        })
    }

    fn try_propagate_type_from_outer_to_child(&mut self, parent_idx: usize, child_idx: usize, parent_hint: Option<ValueType>) -> Result<bool, Box<dyn Error>> {
        if self.nodes[child_idx].resolved_type.is_none() {
            let child_hint = self.get_child_input_hint(parent_idx, child_idx, parent_hint)?;
            if let Some(child_hint) = child_hint {
                return Ok(self.propagate_type_from_outer(child_idx, child_hint)?);
            }
        }

        Ok(false)
    }

    fn propagate_type_from_outer(&mut self, idx: usize, hint: ValueType) -> Result<bool, Box<dyn Error>> {
        let node = &mut self.nodes[idx];
        if node.resolved_type.is_some() { return Ok(false) }

        let mut had_changes;
        match &node.data {
            PackedAnalysisNodeData::TypedValue { .. } |
            PackedAnalysisNodeData::Variable { .. } |
            PackedAnalysisNodeData::FunctionCall { .. } => unreachable!(),
            PackedAnalysisNodeData::UntypedValue { value } => {
                node.data = PackedAnalysisNodeData::TypedValue { value: value.get_resolved_value(hint)? };
                node.resolved_type = Some(hint);
                had_changes = true;
            },
            PackedAnalysisNodeData::UnaryOperation { operator, right_idx } => {
                let right_idx = *right_idx;

                match operator {
                    UnaryOperator::Negate => {
                        had_changes = self.try_propagate_type_from_outer_to_child(idx, right_idx, Some(hint))?;
                    },
                    UnaryOperator::LogicalNot => unreachable!(),
                }
            },
            PackedAnalysisNodeData::BinaryOperation { operator, left_idx, right_idx } => {
                let left_idx = *left_idx;
                let right_idx = *right_idx;

                match operator {
                    BinaryOperator::Mul |
                    BinaryOperator::Div |
                    BinaryOperator::Mod |
                    BinaryOperator::Add |
                    BinaryOperator::Sub |
                    BinaryOperator::Equals |
                    BinaryOperator::NotEquals |
                    BinaryOperator::LesserThanEquals |
                    BinaryOperator::GreaterThanEquals |
                    BinaryOperator::LesserThan |
                    BinaryOperator::GreaterThan => {
                        had_changes = self.try_propagate_type_from_outer_to_child(idx, left_idx, Some(hint))?;
                        had_changes = self.try_propagate_type_from_outer_to_child(idx, right_idx, Some(hint))? || had_changes;
                    },
                    BinaryOperator::LogicalAnd |
                    BinaryOperator::LogicalOr => unreachable!(),
                }
            },
            PackedAnalysisNodeData::Ternary { cond_idx: _, left_idx, right_idx } => {
                let left_idx = *left_idx;
                let right_idx = *right_idx;
                had_changes = self.try_propagate_type_from_outer_to_child(idx, left_idx, Some(hint))?;
                had_changes = self.try_propagate_type_from_outer_to_child(idx, right_idx, Some(hint))? || had_changes;
            },
        };

        Ok(had_changes)
    }

    fn resolve_types_from_outer(&mut self) -> Result<bool, Box<dyn Error>> {
        let mut had_changes = false;
        for idx in (0..self.nodes.len()).rev() {
            let node = &self.nodes[idx];
            match node.resolved_type { Some(_) => continue, None => {} };
            let parent_idx = match node.parent_idx { Some(x) => x, None => continue };
            let hint = self.get_child_input_hint(parent_idx, idx, None)?;
            if let Some(hint) = hint {
                had_changes = self.propagate_type_from_outer(idx, hint)? || had_changes;
            }
        }

        Ok(had_changes)
    }

    fn resolve_types_from_both(&mut self) -> Result<(), Box<dyn Error>> {
        let mut had_changes = true;
        while had_changes {
            had_changes = self.resolve_types_from_inner()?;
            had_changes = self.resolve_types_from_outer()? || had_changes;
        }

        Ok(())
    }

    fn semantic_analysis(&mut self) -> Result<(), Box<dyn Error>> {
        // propagates types from inner (low index) to outer (high index)
        // expressions. outer expressions depend on inner expressions, not the
        // other way around, so if all leaf expressions are typed, then the
        // entire tree should be resolved. if it doesn't get resolved after
        // this, then it means one of the leaf nodes is an untyped value which
        // was not auto-resolved, and we can try to propagate the type from
        // outer to inner if it's an operation that expects operands to have the
        // same type (this is usually the case). if that still fails, it means
        // that all leafs for that operation's subtree are untyped and the
        // fallback types need to be used

        // this gets complicated by the fact that we have untyped values, which
        // depend on what they're being used with, which is usually sibling
        // nodes, meaning that untyped values do kinda sorta depend on
        // non-parent nodes, in a convoluted way, so we end up having to do
        // multiple iterations of type propagation
        self.resolve_types_from_both()?;

        let mut had_fallback = false;
        for i in 0..self.nodes.len() {
            let node = &mut self.nodes[i];
            if node.resolved_type.is_none() {
                if let PackedAnalysisNodeData::UntypedValue { value } = &node.data {
                    let resolved_type = match value {
                        UntypedValue::Float { .. } => {
                            Some(ValueType::F64)
                        },
                        UntypedValue::Integer { inner } => {
                            if *inner <= i32::MAX as u64 {
                                Some(ValueType::I32)
                            } else if *inner <= i64::MAX as u64 {
                                Some(ValueType::I64)
                            } else {
                                // XXX falling back to a u64 is not allowed even
                                //     if it doesn't fit in a i64. the type will
                                //     be unresolved unless something else
                                //     causes it to be resolved
                                None
                            }
                        },
                    };

                    if let Some(resolved_type) = resolved_type {
                        node.data = PackedAnalysisNodeData::TypedValue { value: value.get_resolved_value(resolved_type)? };
                        node.resolved_type = Some(resolved_type);
                        had_fallback = true;
                    }
                }
            }
        }

        if had_fallback {
            self.resolve_types_from_both()?;
        }

        Ok(())
    }

    fn print_node_to_stderr(&self, idx: usize, depth: usize) {
        let node = &self.nodes[idx];
        eprintln!("{}[{}]: {:?}", "  ".repeat(depth), idx, node);

        match &node.data {
            PackedAnalysisNodeData::TypedValue { .. } |
            PackedAnalysisNodeData::UntypedValue { .. } |
            PackedAnalysisNodeData::Variable { .. } => { },
            PackedAnalysisNodeData::FunctionCall { args, fn_spec: _ } => {
                for PackedAnalysisFunctionArg { idx, expected_type: _ } in args {
                    self.print_node_to_stderr(*idx, depth + 1);
                }
            },
            PackedAnalysisNodeData::UnaryOperation { operator: _, right_idx } => {
                self.print_node_to_stderr(*right_idx, depth + 1);
            },
            PackedAnalysisNodeData::BinaryOperation { operator: _, left_idx, right_idx } => {
                self.print_node_to_stderr(*left_idx, depth + 1);
                self.print_node_to_stderr(*right_idx, depth + 1);
            },
            PackedAnalysisNodeData::Ternary { cond_idx, left_idx, right_idx } => {
                self.print_node_to_stderr(*cond_idx, depth + 1);
                self.print_node_to_stderr(*left_idx, depth + 1);
                self.print_node_to_stderr(*right_idx, depth + 1);
            },
        }
    }

    pub fn print_to_stderr(&self) {
        let node_count = self.nodes.len();
        eprintln!("PackedAnalysisTree with {} nodes:", node_count);
        let root_idx = node_count - 1;
        self.print_node_to_stderr(root_idx, 0);
    }

    pub fn get_expr_type(&self) -> Result<ValueType, AnalysisError> {
        let node_count = self.nodes.len();
        if node_count == 0 { return Err(AnalysisError::EmptyAST) }
        let root = &self.nodes[node_count - 1];
        if let Some(expr_type) = root.resolved_type {
            Ok(expr_type)
        } else {
            Err(AnalysisError::BadAnalysis)
        }
    }

    pub fn get_node_type(&self, idx: usize) -> Result<ValueType, AnalysisError> {
        match self.nodes[idx].resolved_type {
            Some(t) => Ok(t),
            None => Err(AnalysisError::BadAnalysis),
        }
    }
}