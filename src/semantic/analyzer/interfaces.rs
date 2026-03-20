use crate::conversion::ConversionKind;
use crate::frontend::ast::{Expression, TypeRef};
use crate::semantic::analyzer::{FunctionAnalyzer, SemanticError};
use crate::semantic::model::{CheckedExpression, CheckedExpressionKind, Type};
use crate::semantic::support::{
    coerce_expression_to_type, resolve_type_ref, validate_runtime_type,
};

impl<'a> FunctionAnalyzer<'a> {
    pub(super) fn analyze_conversion_expression(
        &mut self,
        type_ref: &TypeRef,
        value: &Expression,
    ) -> Result<CheckedExpression, SemanticError> {
        let target_type = resolve_type_ref(type_ref).ok_or_else(|| {
            SemanticError::new(format!(
                "conversion does not support target type `{}`",
                type_ref.render()
            ))
        })?;
        let value = self.analyze_expression(value)?;
        if target_type == Type::Any {
            return coerce_expression_to_type(&Type::Any, value, "conversion to `any`");
        }
        let conversion = match (&target_type, &value.ty) {
            (Type::Slice(element), Type::String) if element.as_ref() == &Type::Byte => {
                ConversionKind::StringToBytes
            }
            (Type::String, source) if source.is_byte_slice() => ConversionKind::BytesToString,
            (Type::Slice(element), _) if element.as_ref() == &Type::Byte => {
                return Err(SemanticError::new(format!(
                    "conversion to `[]byte` requires `string`, found `{}`",
                    value.ty.render()
                )));
            }
            (Type::String, _) => {
                return Err(SemanticError::new(format!(
                    "conversion to `string` requires `[]byte`, found `{}`",
                    value.ty.render()
                )));
            }
            _ => {
                return Err(SemanticError::new(format!(
                    "conversion to `{}` is not supported",
                    target_type.render()
                )));
            }
        };
        Ok(CheckedExpression {
            ty: target_type,
            kind: CheckedExpressionKind::Conversion {
                conversion,
                value: Box::new(value),
            },
        })
    }

    pub(super) fn analyze_type_assertion_expression(
        &mut self,
        target: &Expression,
        asserted_type_ref: &TypeRef,
    ) -> Result<CheckedExpression, SemanticError> {
        let value = self.analyze_expression(target)?;
        if value.ty != Type::Any {
            return Err(SemanticError::new(format!(
                "type assertion requires interface operand, found `{}`",
                value.ty.render()
            )));
        }

        let asserted_type = resolve_type_ref(asserted_type_ref).ok_or_else(|| {
            SemanticError::new(format!(
                "type assertion does not support target type `{}`",
                asserted_type_ref.render()
            ))
        })?;
        validate_runtime_type(&asserted_type, "type assertion target")?;

        Ok(CheckedExpression {
            ty: asserted_type.clone(),
            kind: CheckedExpressionKind::TypeAssertion {
                value: Box::new(value),
                asserted_type,
            },
        })
    }
}
