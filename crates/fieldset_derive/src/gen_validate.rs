use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

use crate::parse::{
  ConditionLiteral, CrossValidateRule, FieldInfo, FieldType, NumericLit, OneOfItem, ValidateAttr,
};

/// Generate the body of `fn validate(&self) -> Result<(), FieldsetViolations>`.
pub fn gen_validate(
  fields: &[FieldInfo],
  cross_rules: &[CrossValidateRule],
  struct_break_on_failure: bool,
) -> syn::Result<TokenStream> {
  let field_checks: Vec<TokenStream> = fields
    .iter()
    .filter(|f| !f.validations.is_empty() || f.is_nested_validate)
    .map(|f| gen_field_validate(f, struct_break_on_failure))
    .collect();

  let cross_checks: Vec<TokenStream> = cross_rules
    .iter()
    .map(|rule| gen_cross_validate(rule, fields))
    .collect::<syn::Result<Vec<_>>>()?;

  Ok(quote! {
    fn validate(&self) -> ::core::result::Result<(), walrs_validation::FieldsetViolations> {
      let mut violations = walrs_validation::FieldsetViolations::new();
      #(#field_checks)*
      #(#cross_checks)*
      violations.into()
    }
  })
}

/// Find a field by ident; error if missing. Returns its type for codegen decisions.
fn lookup_field<'a>(fields: &'a [FieldInfo], name: &Ident) -> syn::Result<&'a FieldInfo> {
  fields.iter().find(|f| &f.ident == name).ok_or_else(|| {
    syn::Error::new(
      name.span(),
      format!("cross_validate references unknown field `{name}`"),
    )
  })
}

/// Emit an expression of type `bool` that is `true` when `self.<field>` carries a value.
///
/// String presence checks use `trim().is_empty()` to match `walrs_validation`'s
/// `IsEmpty for String` and `ValueExt::is_empty_value()` semantics — a
/// whitespace-only string is treated as empty here, just like field-level
/// `required` validation and the dynamic cross-field rules.
///
/// - `Option<T>` → `self.field.is_some()` (and for Option<String>, also non-blank)
/// - `String` → `!self.field.trim().is_empty()`
/// - other scalars (numeric/bool/char) → always `true`
fn emit_has_value(field: &FieldInfo) -> TokenStream {
  let name = &field.ident;
  match &field.ty {
    FieldType::String => quote! { !self.#name.trim().is_empty() },
    FieldType::OptionString => {
      quote! { self.#name.as_ref().map(|s| !s.trim().is_empty()).unwrap_or(false) }
    }
    FieldType::OptionBool
    | FieldType::OptionChar
    | FieldType::OptionNumeric(_)
    | FieldType::OptionOther(_) => quote! { self.#name.is_some() },
    _ => quote! { true },
  }
}

/// Emit an expression of type `bool` that compares `self.<field>` against `lit`.
fn emit_eq_literal(field: &FieldInfo, lit: &ConditionLiteral) -> TokenStream {
  let name = &field.ident;
  match (&field.ty, lit) {
    (FieldType::String, ConditionLiteral::Str(s)) => quote! { self.#name == #s },
    (FieldType::OptionString, ConditionLiteral::Str(s)) => {
      quote! { self.#name.as_deref() == ::core::option::Option::Some(#s) }
    }
    (FieldType::Bool, ConditionLiteral::Bool(b)) => quote! { self.#name == #b },
    (FieldType::OptionBool, ConditionLiteral::Bool(b)) => {
      quote! { self.#name == ::core::option::Option::Some(#b) }
    }
    (FieldType::Numeric(_), ConditionLiteral::Int(v)) => {
      let lit = proc_macro2::Literal::i128_unsuffixed(*v);
      quote! { self.#name == #lit }
    }
    (FieldType::OptionNumeric(_), ConditionLiteral::Int(v)) => {
      let lit = proc_macro2::Literal::i128_unsuffixed(*v);
      quote! { self.#name == ::core::option::Option::Some(#lit) }
    }
    // Fallback: fall through to direct equality and let rustc surface a type
    // mismatch at the call site rather than silently codegen the wrong thing.
    (_, ConditionLiteral::Str(s)) => quote! { self.#name == #s },
    (_, ConditionLiteral::Bool(b)) => quote! { self.#name == #b },
    (_, ConditionLiteral::Int(v)) => {
      let lit = proc_macro2::Literal::i128_unsuffixed(*v);
      quote! { self.#name == #lit }
    }
  }
}

fn gen_cross_validate(rule: &CrossValidateRule, fields: &[FieldInfo]) -> syn::Result<TokenStream> {
  match rule {
    CrossValidateRule::Custom(path) => Ok(quote! {
      if let Err(violation) = #path(&self) {
        violations.add_form_violation(violation);
      }
    }),
    CrossValidateRule::FieldsEqual { field_a, field_b } => {
      let _ = lookup_field(fields, field_a)?;
      let _ = lookup_field(fields, field_b)?;
      let a_str = field_a.to_string();
      let b_str = field_b.to_string();
      Ok(quote! {
        if self.#field_a != self.#field_b {
          violations.add_form_violation(walrs_validation::Violation::new(
            walrs_validation::ViolationType::NotEqual,
            ::std::format!("FieldsEqual: {} and {} must be equal", #a_str, #b_str),
          ));
        }
      })
    }
    CrossValidateRule::RequiredIf {
      field,
      condition_field,
      condition,
    } => {
      let field_info = lookup_field(fields, field)?;
      let cond_info = lookup_field(fields, condition_field)?;
      let has_value = emit_has_value(field_info);
      let cond_check = emit_eq_literal(cond_info, condition);
      let f_str = field.to_string();
      let c_str = condition_field.to_string();
      Ok(quote! {
        if (#cond_check) && !(#has_value) {
          violations.add_form_violation(walrs_validation::Violation::new(
            walrs_validation::ViolationType::ValueMissing,
            ::std::format!(
              "RequiredIf: {} is required when condition is met on {}",
              #f_str, #c_str
            ),
          ));
        }
      })
    }
    CrossValidateRule::RequiredUnless {
      field,
      condition_field,
      condition,
    } => {
      let field_info = lookup_field(fields, field)?;
      let cond_info = lookup_field(fields, condition_field)?;
      let has_value = emit_has_value(field_info);
      let cond_check = emit_eq_literal(cond_info, condition);
      let f_str = field.to_string();
      let c_str = condition_field.to_string();
      Ok(quote! {
        if !(#cond_check) && !(#has_value) {
          violations.add_form_violation(walrs_validation::Violation::new(
            walrs_validation::ViolationType::ValueMissing,
            ::std::format!(
              "RequiredUnless: {} is required unless condition is met on {}",
              #f_str, #c_str
            ),
          ));
        }
      })
    }
    CrossValidateRule::OneOfRequired { fields: names } => {
      let checks: Vec<TokenStream> = names
        .iter()
        .map(|n| lookup_field(fields, n).map(emit_has_value))
        .collect::<syn::Result<_>>()?;
      let names_csv = names
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(", ");
      Ok(quote! {
        if !((#(#checks)||*)) {
          violations.add_form_violation(walrs_validation::Violation::new(
            walrs_validation::ViolationType::ValueMissing,
            ::std::format!("OneOfRequired: At least one of {} is required", #names_csv),
          ));
        }
      })
    }
    CrossValidateRule::MutuallyExclusive { fields: names } => {
      let checks: Vec<TokenStream> = names
        .iter()
        .map(|n| lookup_field(fields, n).map(emit_has_value))
        .collect::<syn::Result<_>>()?;
      let names_csv = names
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(", ");
      Ok(quote! {
        {
          let __filled: usize = 0 #(+ if #checks { 1 } else { 0 })*;
          if __filled > 1 {
            violations.add_form_violation(walrs_validation::Violation::new(
              walrs_validation::ViolationType::CustomError,
              ::std::format!("MutuallyExclusive: Only one of {} can have a value", #names_csv),
            ));
          }
        }
      })
    }
    CrossValidateRule::DependentRequired {
      trigger,
      dependents,
    } => {
      let trigger_info = lookup_field(fields, trigger)?;
      let trigger_check = emit_has_value(trigger_info);
      let dep_arms: Vec<TokenStream> = dependents
        .iter()
        .map(|n| {
          let info = lookup_field(fields, n)?;
          let has = emit_has_value(info);
          let n_str = n.to_string();
          let t_str = trigger.to_string();
          Ok(quote! {
            if !(#has) {
              violations.add_form_violation(walrs_validation::Violation::new(
                walrs_validation::ViolationType::ValueMissing,
                ::std::format!(
                  "DependentRequired: {} is required when {} is provided",
                  #n_str, #t_str
                ),
              ));
            }
          })
        })
        .collect::<syn::Result<_>>()?;
      Ok(quote! {
        if #trigger_check {
          #(#dep_arms)*
        }
      })
    }
  }
}

fn gen_field_validate(field: &FieldInfo, struct_break: bool) -> TokenStream {
  let field_name = &field.ident;
  let field_name_str = field_name.to_string();

  let break_on_failure = field.break_on_failure_override.unwrap_or(struct_break);

  let break_check = if break_on_failure {
    quote! {
      if !violations.is_empty() {
        return ::core::result::Result::Err(violations);
      }
    }
  } else {
    quote! {}
  };

  // Handle nested validation
  if field.is_nested_validate {
    return gen_nested_validate(field, &break_check);
  }

  // Build rule expression
  let rules = build_rules(field);
  if rules.is_none() {
    return quote! {};
  }
  // SAFETY: `rules` is checked for `None` above
  let rule_expr = rules.unwrap();

  match &field.ty {
    FieldType::String => {
      quote! {
        {
          let rule = #rule_expr;
          if let ::core::result::Result::Err(violation) =
            walrs_validation::ValidateRef::validate_ref(&rule, self.#field_name.as_str())
          {
            violations.add(#field_name_str, violation);
            #break_check
          }
        }
      }
    }
    FieldType::Numeric(_) | FieldType::Bool | FieldType::Char => {
      quote! {
        {
          let rule = #rule_expr;
          if let ::core::result::Result::Err(violation) =
            walrs_validation::ValidateRef::validate_ref(&rule, &self.#field_name)
          {
            violations.add(#field_name_str, violation);
            #break_check
          }
        }
      }
    }
    FieldType::OptionString => {
      let has_required = field
        .validations
        .iter()
        .any(|v| matches!(v, ValidateAttr::Required));
      if has_required {
        quote! {
          {
            let rule = #rule_expr;
            match self.#field_name.as_ref() {
              ::core::option::Option::Some(inner) => {
                if let ::core::result::Result::Err(violation) =
                  walrs_validation::ValidateRef::validate_ref(&rule, inner.as_str())
                {
                  violations.add(#field_name_str, violation);
                  #break_check
                }
              }
              ::core::option::Option::None => {
                violations.add(#field_name_str, walrs_validation::Violation::value_missing());
                #break_check
              }
            }
          }
        }
      } else {
        quote! {
          {
            let rule = #rule_expr;
            if let ::core::option::Option::Some(inner) = self.#field_name.as_ref() {
              if let ::core::result::Result::Err(violation) =
                walrs_validation::ValidateRef::validate_ref(&rule, inner.as_str())
              {
                violations.add(#field_name_str, violation);
                #break_check
              }
            }
          }
        }
      }
    }
    FieldType::OptionNumeric(_) | FieldType::OptionBool | FieldType::OptionChar => {
      let has_required = field
        .validations
        .iter()
        .any(|v| matches!(v, ValidateAttr::Required));
      if has_required {
        quote! {
          {
            let rule = #rule_expr;
            match self.#field_name.as_ref() {
              ::core::option::Option::Some(inner) => {
                if let ::core::result::Result::Err(violation) =
                  walrs_validation::ValidateRef::validate_ref(&rule, inner)
                {
                  violations.add(#field_name_str, violation);
                  #break_check
                }
              }
              ::core::option::Option::None => {
                violations.add(#field_name_str, walrs_validation::Violation::value_missing());
                #break_check
              }
            }
          }
        }
      } else {
        quote! {
          {
            let rule = #rule_expr;
            if let ::core::option::Option::Some(inner) = self.#field_name.as_ref() {
              if let ::core::result::Result::Err(violation) =
                walrs_validation::ValidateRef::validate_ref(&rule, inner)
              {
                violations.add(#field_name_str, violation);
                #break_check
              }
            }
          }
        }
      }
    }
    FieldType::Other(_) | FieldType::OptionOther(_) => {
      // For unknown types, attempt ValidateRef
      quote! {
        {
          let rule = #rule_expr;
          if let ::core::result::Result::Err(violation) =
            walrs_validation::ValidateRef::validate_ref(&rule, &self.#field_name)
          {
            violations.add(#field_name_str, violation);
            #break_check
          }
        }
      }
    }
  }
}

fn gen_nested_validate(field: &FieldInfo, break_check: &TokenStream) -> TokenStream {
  let field_name = &field.ident;
  let field_name_str = field_name.to_string();

  match &field.ty {
    FieldType::OptionOther(_) | FieldType::OptionString => {
      let has_required = field
        .validations
        .iter()
        .any(|v| matches!(v, ValidateAttr::Required));
      if has_required {
        quote! {
          match self.#field_name.as_ref() {
            ::core::option::Option::Some(inner) => {
              if let ::core::result::Result::Err(nested_violations) =
                walrs_fieldfilter::Fieldset::validate(inner)
              {
                violations.merge_prefixed(#field_name_str, nested_violations);
                #break_check
              }
            }
            ::core::option::Option::None => {
              violations.add(#field_name_str, walrs_validation::Violation::value_missing());
              #break_check
            }
          }
        }
      } else {
        quote! {
          if let ::core::option::Option::Some(inner) = self.#field_name.as_ref() {
            if let ::core::result::Result::Err(nested_violations) =
              walrs_fieldfilter::Fieldset::validate(inner)
            {
              violations.merge_prefixed(#field_name_str, nested_violations);
              #break_check
            }
          }
        }
      }
    }
    _ => {
      quote! {
        if let ::core::result::Result::Err(nested_violations) =
          walrs_fieldfilter::Fieldset::validate(&self.#field_name)
        {
          violations.merge_prefixed(#field_name_str, nested_violations);
          #break_check
        }
      }
    }
  }
}

/// Build the rule expression tokens for a field.
fn build_rules(field: &FieldInfo) -> Option<TokenStream> {
  let rule_type = match &field.ty {
    FieldType::String | FieldType::OptionString => quote! { String },
    FieldType::Numeric(id) | FieldType::OptionNumeric(id) => quote! { #id },
    FieldType::Bool | FieldType::OptionBool => quote! { bool },
    FieldType::Char | FieldType::OptionChar => quote! { char },
    FieldType::Other(_) | FieldType::OptionOther(_) => return None,
  };

  // Separate message/locale modifiers from actual rules
  let mut message: Option<String> = None;
  let mut message_fn: Option<&syn::Path> = None;
  let mut locale: Option<String> = None;
  let mut rule_attrs: Vec<&ValidateAttr> = Vec::new();

  for attr in &field.validations {
    match attr {
      ValidateAttr::Message(m) => message = Some(m.clone()),
      ValidateAttr::MessageFn(p) => message_fn = Some(p),
      ValidateAttr::Locale(l) => locale = Some(l.clone()),
      _ => rule_attrs.push(attr),
    }
  }

  if rule_attrs.is_empty() {
    return None;
  }

  let individual_rules: Vec<TokenStream> = rule_attrs
    .iter()
    .map(|attr| attr_to_rule_token(attr, &rule_type))
    .collect();

  let base_rule = if individual_rules.len() == 1 {
    // SAFETY: length checked to be exactly 1
    individual_rules.into_iter().next().unwrap()
  } else {
    quote! {
      walrs_validation::Rule::<#rule_type>::All(::std::vec![#(#individual_rules),*])
    }
  };

  // Apply message/locale wrappers
  let wrapped = apply_message_wrappers(base_rule, &message, &message_fn, &locale);

  Some(wrapped)
}

fn attr_to_rule_token(attr: &ValidateAttr, rule_type: &TokenStream) -> TokenStream {
  match attr {
    ValidateAttr::Required => quote! { walrs_validation::Rule::<#rule_type>::Required },
    ValidateAttr::MinLength(n) => {
      quote! { walrs_validation::Rule::<#rule_type>::MinLength(#n) }
    }
    ValidateAttr::MaxLength(n) => {
      quote! { walrs_validation::Rule::<#rule_type>::MaxLength(#n) }
    }
    ValidateAttr::ExactLength(n) => {
      quote! { walrs_validation::Rule::<#rule_type>::ExactLength(#n) }
    }
    ValidateAttr::Email => {
      quote! { walrs_validation::Rule::<#rule_type>::Email(::core::default::Default::default()) }
    }
    ValidateAttr::Url => {
      quote! { walrs_validation::Rule::<#rule_type>::Url(::core::default::Default::default()) }
    }
    ValidateAttr::Uri => {
      quote! { walrs_validation::Rule::<#rule_type>::Uri(::core::default::Default::default()) }
    }
    ValidateAttr::Ip => {
      quote! { walrs_validation::Rule::<#rule_type>::Ip(::core::default::Default::default()) }
    }
    ValidateAttr::Hostname => {
      quote! { walrs_validation::Rule::<#rule_type>::Hostname(::core::default::Default::default()) }
    }
    ValidateAttr::Pattern(pat) => {
      // Pattern is validated at macro expansion time in parse.rs
      quote! {
        walrs_validation::Rule::<#rule_type>::Pattern(
          walrs_validation::CompiledPattern::try_from(#pat)
            .expect("regex validated at macro expansion time")
        )
      }
    }
    ValidateAttr::Min(n) => {
      let lit = numeric_lit_token(n);
      quote! { walrs_validation::Rule::<#rule_type>::Min(#lit) }
    }
    ValidateAttr::Max(n) => {
      let lit = numeric_lit_token(n);
      quote! { walrs_validation::Rule::<#rule_type>::Max(#lit) }
    }
    ValidateAttr::Range { min, max } => {
      let min_lit = numeric_lit_token(min);
      let max_lit = numeric_lit_token(max);
      quote! { walrs_validation::Rule::<#rule_type>::Range { min: #min_lit, max: #max_lit } }
    }
    ValidateAttr::Step(n) => {
      let lit = numeric_lit_token(n);
      quote! { walrs_validation::Rule::<#rule_type>::Step(#lit) }
    }
    ValidateAttr::OneOf(items) => {
      let item_tokens: Vec<TokenStream> = items.iter().map(one_of_item_token).collect();
      quote! { walrs_validation::Rule::<#rule_type>::OneOf(::std::vec![#(#item_tokens),*]) }
    }
    ValidateAttr::Custom(path) => {
      quote! {
        walrs_validation::Rule::<#rule_type>::Custom(::std::sync::Arc::new(#path))
      }
    }
    // Message/MessageFn/Locale are handled separately
    ValidateAttr::Message(_) | ValidateAttr::MessageFn(_) | ValidateAttr::Locale(_) => {
      quote! {}
    }
  }
}

fn numeric_lit_token(n: &NumericLit) -> TokenStream {
  match n {
    NumericLit::Int(v) => {
      // Use the raw integer value; Rust will infer the type
      let lit = proc_macro2::Literal::i128_unsuffixed(*v);
      quote! { #lit }
    }
    NumericLit::Float(v) => {
      let lit = proc_macro2::Literal::f64_unsuffixed(*v);
      quote! { #lit }
    }
  }
}

fn one_of_item_token(item: &OneOfItem) -> TokenStream {
  match item {
    OneOfItem::Str(s) => quote! { #s.to_string() },
    OneOfItem::Int(v) => {
      let lit = proc_macro2::Literal::i128_unsuffixed(*v);
      quote! { #lit }
    }
    OneOfItem::Float(v) => {
      let lit = proc_macro2::Literal::f64_unsuffixed(*v);
      quote! { #lit }
    }
  }
}

fn apply_message_wrappers(
  base: TokenStream,
  message: &Option<String>,
  message_fn: &Option<&syn::Path>,
  locale: &Option<String>,
) -> TokenStream {
  let mut result = base;

  if let Some(msg) = message {
    result = quote! { (#result).with_message(#msg) };
  }

  if let Some(path) = message_fn {
    result = quote! { (#result).with_message_provider(#path, ::core::option::Option::None) };
  }

  if let Some(loc) = locale {
    result = quote! { (#result).with_locale(#loc) };
  }

  result
}
