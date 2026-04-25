//! Async codegen for `FieldsetAsync::validate_async`.
//!
//! Mirrors `gen_validate.rs` but emits an `async fn validate_async` body. The
//! sync rules are still built into a `Rule` and evaluated via
//! `ValidateRefAsync::validate_ref_async` (which handles them inline). Each
//! `custom_async = "path::fn"` attribute is invoked directly as
//! `path(value).await` and accumulated into the same `FieldsetViolations`.

use proc_macro2::TokenStream;
use quote::quote;

use crate::parse::{FieldInfo, FieldType, NumericLit, OneOfItem, ValidateAttr};

/// Generate the body of `fn validate_async(&self) -> ... + Send`.
pub fn gen_validate_async(
  fields: &[FieldInfo],
  cross_fns: &[syn::Path],
  struct_break_on_failure: bool,
) -> TokenStream {
  let field_checks: Vec<TokenStream> = fields
    .iter()
    .filter(|f| !f.validations.is_empty() || f.is_nested_validate)
    .map(|f| gen_field_validate_async(f, struct_break_on_failure))
    .collect();

  let cross_checks: Vec<TokenStream> = cross_fns
    .iter()
    .map(|path| {
      quote! {
        if let Err(violation) = #path(&self) {
          violations.add_form_violation(violation);
        }
      }
    })
    .collect();

  quote! {
    fn validate_async(
      &self,
    ) -> impl ::core::future::Future<Output = ::core::result::Result<(), walrs_validation::FieldsetViolations>> + Send {
      async move {
        let mut violations = walrs_validation::FieldsetViolations::new();
        #(#field_checks)*
        #(#cross_checks)*
        violations.into()
      }
    }
  }
}

fn gen_field_validate_async(field: &FieldInfo, struct_break: bool) -> TokenStream {
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

  // Nested fieldset delegates to the nested type's FieldsetAsync impl.
  if field.is_nested_validate {
    return gen_nested_validate_async(field, &break_check);
  }

  let sync_block = gen_sync_rule_block(field, &break_check);
  let async_block = gen_custom_async_block(field, &field_name_str, &break_check);

  quote! {
    #sync_block
    #async_block
  }
}

/// Emit the synchronous rule evaluation (calls `validate_ref_async` on a `Rule`).
fn gen_sync_rule_block(field: &FieldInfo, break_check: &TokenStream) -> TokenStream {
  let field_name = &field.ident;
  let field_name_str = field_name.to_string();

  let rules = build_rules(field);
  let Some(rule_expr) = rules else {
    return quote! {};
  };

  match &field.ty {
    FieldType::String => {
      quote! {
        {
          let rule = #rule_expr;
          if let ::core::result::Result::Err(violation) =
            walrs_validation::ValidateRefAsync::validate_ref_async(&rule, self.#field_name.as_str()).await
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
            walrs_validation::ValidateRefAsync::validate_ref_async(&rule, &self.#field_name).await
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
                  walrs_validation::ValidateRefAsync::validate_ref_async(&rule, inner.as_str()).await
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
                walrs_validation::ValidateRefAsync::validate_ref_async(&rule, inner.as_str()).await
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
                  walrs_validation::ValidateRefAsync::validate_ref_async(&rule, inner).await
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
                walrs_validation::ValidateRefAsync::validate_ref_async(&rule, inner).await
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
      quote! {
        {
          let rule = #rule_expr;
          if let ::core::result::Result::Err(violation) =
            walrs_validation::ValidateRefAsync::validate_ref_async(&rule, &self.#field_name).await
          {
            violations.add(#field_name_str, violation);
            #break_check
          }
        }
      }
    }
  }
}

/// Emit invocations for each `custom_async = "..."` validator.
fn gen_custom_async_block(
  field: &FieldInfo,
  field_name_str: &str,
  break_check: &TokenStream,
) -> TokenStream {
  let field_name = &field.ident;

  let async_paths: Vec<&syn::Path> = field
    .validations
    .iter()
    .filter_map(|v| match v {
      ValidateAttr::CustomAsync(p) => Some(p),
      _ => None,
    })
    .collect();

  if async_paths.is_empty() {
    return quote! {};
  }

  let value_expr: TokenStream = match &field.ty {
    FieldType::String => quote! { self.#field_name.as_str() },
    FieldType::OptionString => quote! { self.#field_name.as_deref() },
    _ => quote! { &self.#field_name },
  };

  let calls: Vec<TokenStream> = async_paths
    .into_iter()
    .map(|path| {
      quote! {
        if let ::core::result::Result::Err(violation) = #path(#value_expr).await {
          violations.add(#field_name_str, violation);
          #break_check
        }
      }
    })
    .collect();

  quote! {
    #(#calls)*
  }
}

fn gen_nested_validate_async(field: &FieldInfo, break_check: &TokenStream) -> TokenStream {
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
                walrs_fieldfilter::FieldsetAsync::validate_async(inner).await
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
              walrs_fieldfilter::FieldsetAsync::validate_async(inner).await
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
          walrs_fieldfilter::FieldsetAsync::validate_async(&self.#field_name).await
        {
          violations.merge_prefixed(#field_name_str, nested_violations);
          #break_check
        }
      }
    }
  }
}

// ---------------------------------------------------------------------------
// Rule expression construction (mirrors gen_validate.rs but skips CustomAsync)
// ---------------------------------------------------------------------------

fn build_rules(field: &FieldInfo) -> Option<TokenStream> {
  let rule_type = match &field.ty {
    FieldType::String | FieldType::OptionString => quote! { String },
    FieldType::Numeric(id) | FieldType::OptionNumeric(id) => quote! { #id },
    FieldType::Bool | FieldType::OptionBool => quote! { bool },
    FieldType::Char | FieldType::OptionChar => quote! { char },
    FieldType::Other(_) | FieldType::OptionOther(_) => return None,
  };

  let mut message: Option<String> = None;
  let mut message_fn: Option<&syn::Path> = None;
  let mut locale: Option<String> = None;
  let mut rule_attrs: Vec<&ValidateAttr> = Vec::new();

  for attr in &field.validations {
    match attr {
      ValidateAttr::Message(m) => message = Some(m.clone()),
      ValidateAttr::MessageFn(p) => message_fn = Some(p),
      ValidateAttr::Locale(l) => locale = Some(l.clone()),
      // CustomAsync is handled out-of-band by gen_custom_async_block.
      ValidateAttr::CustomAsync(_) => {}
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
    individual_rules.into_iter().next().unwrap()
  } else {
    quote! {
      walrs_validation::Rule::<#rule_type>::All(::std::vec![#(#individual_rules),*])
    }
  };

  Some(apply_message_wrappers(
    base_rule,
    &message,
    &message_fn,
    &locale,
  ))
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
    ValidateAttr::Message(_)
    | ValidateAttr::MessageFn(_)
    | ValidateAttr::Locale(_)
    | ValidateAttr::CustomAsync(_) => {
      quote! {}
    }
  }
}

fn numeric_lit_token(n: &NumericLit) -> TokenStream {
  match n {
    NumericLit::Int(v) => {
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
