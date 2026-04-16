//! Code generation for FormData bridge (`into_form_data`, `try_from_form_data`).

use crate::parse::{FieldInfo, FieldType};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Generate `impl From<&T> for walrs_form::FormData` if requested.
pub fn gen_into_form_data(
  struct_name: &Ident,
  field_infos: &[FieldInfo],
  impl_generics: &syn::ImplGenerics,
  ty_generics: &syn::TypeGenerics,
  where_clause: Option<&syn::WhereClause>,
) -> TokenStream {
  let field_conversions: Vec<TokenStream> = field_infos
    .iter()
    .map(|field| {
      let field_name = &field.ident;
      let field_name_str = field_name.to_string();
      
      match &field.ty {
        FieldType::String => quote! {
          data.insert(#field_name_str, walrs_validation::Value::Str(value.#field_name.clone()));
        },
        FieldType::Bool => quote! {
          data.insert(#field_name_str, walrs_validation::Value::Bool(value.#field_name));
        },
        FieldType::Char => quote! {
          data.insert(#field_name_str, walrs_validation::Value::Str(value.#field_name.to_string()));
        },
        FieldType::Numeric(num_type) => {
          // Cast to the appropriate Value-compatible type
          let num_type_str = num_type.to_string();
          match num_type_str.as_str() {
            "i8" | "i16" | "i32" => quote! {
              data.insert(#field_name_str, walrs_validation::Value::I64(value.#field_name as i64));
            },
            "i64" | "isize" => quote! {
              data.insert(#field_name_str, walrs_validation::Value::I64(value.#field_name as i64));
            },
            "i128" => quote! {
              data.insert(#field_name_str, walrs_validation::Value::Str(value.#field_name.to_string()));
            },
            "u8" | "u16" | "u32" => quote! {
              data.insert(#field_name_str, walrs_validation::Value::U64(value.#field_name as u64));
            },
            "u64" | "usize" => quote! {
              data.insert(#field_name_str, walrs_validation::Value::U64(value.#field_name as u64));
            },
            "u128" => quote! {
              data.insert(#field_name_str, walrs_validation::Value::Str(value.#field_name.to_string()));
            },
            "f32" => quote! {
              data.insert(#field_name_str, walrs_validation::Value::F64(value.#field_name as f64));
            },
            "f64" => quote! {
              data.insert(#field_name_str, walrs_validation::Value::F64(value.#field_name));
            },
            _ => quote! {
              data.insert(#field_name_str, walrs_validation::Value::from(value.#field_name));
            },
          }
        },
        FieldType::OptionString => quote! {
          if let Some(ref val) = value.#field_name {
            data.insert(#field_name_str, walrs_validation::Value::Str(val.clone()));
          } else {
            data.insert(#field_name_str, walrs_validation::Value::Null);
          }
        },
        FieldType::OptionBool => quote! {
          if let Some(val) = value.#field_name {
            data.insert(#field_name_str, walrs_validation::Value::Bool(val));
          } else {
            data.insert(#field_name_str, walrs_validation::Value::Null);
          }
        },
        FieldType::OptionChar => quote! {
          if let Some(val) = value.#field_name {
            data.insert(#field_name_str, walrs_validation::Value::Str(val.to_string()));
          } else {
            data.insert(#field_name_str, walrs_validation::Value::Null);
          }
        },
        FieldType::OptionNumeric(num_type) => {
          let num_type_str = num_type.to_string();
          match num_type_str.as_str() {
            "i8" | "i16" | "i32" | "i64" | "isize" => quote! {
              if let Some(val) = value.#field_name {
                data.insert(#field_name_str, walrs_validation::Value::I64(val as i64));
              } else {
                data.insert(#field_name_str, walrs_validation::Value::Null);
              }
            },
            "i128" => quote! {
              if let Some(val) = value.#field_name {
                data.insert(#field_name_str, walrs_validation::Value::Str(val.to_string()));
              } else {
                data.insert(#field_name_str, walrs_validation::Value::Null);
              }
            },
            "u8" | "u16" | "u32" | "u64" | "usize" => quote! {
              if let Some(val) = value.#field_name {
                data.insert(#field_name_str, walrs_validation::Value::U64(val as u64));
              } else {
                data.insert(#field_name_str, walrs_validation::Value::Null);
              }
            },
            "u128" => quote! {
              if let Some(val) = value.#field_name {
                data.insert(#field_name_str, walrs_validation::Value::Str(val.to_string()));
              } else {
                data.insert(#field_name_str, walrs_validation::Value::Null);
              }
            },
            "f32" | "f64" => quote! {
              if let Some(val) = value.#field_name {
                data.insert(#field_name_str, walrs_validation::Value::F64(val as f64));
              } else {
                data.insert(#field_name_str, walrs_validation::Value::Null);
              }
            },
            _ => quote! {
              if let Some(val) = value.#field_name {
                data.insert(#field_name_str, walrs_validation::Value::from(val));
              } else {
                data.insert(#field_name_str, walrs_validation::Value::Null);
              }
            },
          }
        },
        FieldType::Other(_) | FieldType::OptionOther(_) => {
          // For nested types, we skip them for now or could try to recursively convert
          // if they also implement into_form_data. For simplicity, we'll skip.
          quote! {
            // Nested type conversion not yet implemented for #field_name_str
          }
        }
      }
    })
    .collect();

  quote! {
    impl #impl_generics From<&#struct_name #ty_generics> for walrs_form::FormData #where_clause {
      fn from(value: &#struct_name #ty_generics) -> Self {
        let mut data = walrs_form::FormData::new();
        #(#field_conversions)*
        data
      }
    }
  }
}

/// Generate `impl TryFrom<walrs_form::FormData> for T` if requested.
pub fn gen_try_from_form_data(
  struct_name: &Ident,
  field_infos: &[FieldInfo],
  impl_generics: &syn::ImplGenerics,
  ty_generics: &syn::TypeGenerics,
  where_clause: Option<&syn::WhereClause>,
) -> TokenStream {
  let field_extractions: Vec<TokenStream> = field_infos
    .iter()
    .map(|field| {
      let field_name = &field.ident;
      let field_name_str = field_name.to_string();

      match &field.ty {
        FieldType::String => quote! {
          let #field_name = match data.get_direct(#field_name_str) {
            Some(walrs_validation::Value::Str(s)) => s.clone(),
            Some(_) => {
              violations.add(
                #field_name_str,
                walrs_validation::Violation::new(
                  walrs_validation::ViolationType::TypeMismatch,
                  "Expected string"
                )
              );
              String::new()
            }
            None => {
              violations.add(#field_name_str, walrs_validation::Violation::value_missing());
              String::new()
            }
          };
        },
        FieldType::Bool => quote! {
          let #field_name = match data.get_direct(#field_name_str) {
            Some(walrs_validation::Value::Bool(b)) => *b,
            Some(_) => {
              violations.add(
                #field_name_str,
                walrs_validation::Violation::new(
                  walrs_validation::ViolationType::TypeMismatch,
                  "Expected boolean"
                )
              );
              false
            }
            None => {
              violations.add(#field_name_str, walrs_validation::Violation::value_missing());
              false
            }
          };
        },
        FieldType::Char => quote! {
          let #field_name = match data.get_direct(#field_name_str) {
            Some(walrs_validation::Value::Str(s)) => {
              if s.chars().count() == 1 {
                // SAFETY: `s.chars().count() == 1` guarantees exactly one char
                s.chars().next().unwrap()
              } else {
                violations.add(
                  #field_name_str,
                  walrs_validation::Violation::new(
                    walrs_validation::ViolationType::TypeMismatch,
                    "Expected single character"
                  )
                );
                '\0'
              }
            }
            Some(_) => {
              violations.add(
                #field_name_str,
                walrs_validation::Violation::new(
                  walrs_validation::ViolationType::TypeMismatch,
                  "Expected string"
                )
              );
              '\0'
            }
            None => {
              violations.add(#field_name_str, walrs_validation::Violation::value_missing());
              '\0'
            }
          };
        },
        FieldType::Numeric(num_type) => {
          gen_numeric_extraction(field_name, &field_name_str, num_type, false)
        }
        FieldType::OptionString => quote! {
          let #field_name = match data.get_direct(#field_name_str) {
            Some(walrs_validation::Value::Str(s)) => Some(s.clone()),
            Some(walrs_validation::Value::Null) | None => None,
            Some(_) => {
              violations.add(
                #field_name_str,
                walrs_validation::Violation::new(
                  walrs_validation::ViolationType::TypeMismatch,
                  "Expected string or null"
                )
              );
              None
            }
          };
        },
        FieldType::OptionBool => quote! {
          let #field_name = match data.get_direct(#field_name_str) {
            Some(walrs_validation::Value::Bool(b)) => Some(*b),
            Some(walrs_validation::Value::Null) | None => None,
            Some(_) => {
              violations.add(
                #field_name_str,
                walrs_validation::Violation::new(
                  walrs_validation::ViolationType::TypeMismatch,
                  "Expected boolean or null"
                )
              );
              None
            }
          };
        },
        FieldType::OptionChar => quote! {
          let #field_name = match data.get_direct(#field_name_str) {
            Some(walrs_validation::Value::Str(s)) => {
              // Use chars().count() instead of len() for correct UTF-8 handling
              if s.chars().count() == 1 {
                s.chars().next()
              } else {
                violations.add(
                  #field_name_str,
                  walrs_validation::Violation::new(
                    walrs_validation::ViolationType::TypeMismatch,
                    "Expected single character"
                  )
                );
                None
              }
            }
            Some(walrs_validation::Value::Null) | None => None,
            Some(_) => {
              violations.add(
                #field_name_str,
                walrs_validation::Violation::new(
                  walrs_validation::ViolationType::TypeMismatch,
                  "Expected string or null"
                )
              );
              None
            }
          };
        },
        FieldType::OptionNumeric(num_type) => {
          gen_numeric_extraction(field_name, &field_name_str, num_type, true)
        }
        FieldType::Other(_) | FieldType::OptionOther(_) => {
          // For nested types, we skip them for now
          quote! {
            let #field_name = Default::default();
          }
        }
      }
    })
    .collect();

  let field_names: Vec<_> = field_infos.iter().map(|f| &f.ident).collect();

  quote! {
    impl #impl_generics TryFrom<walrs_form::FormData> for #struct_name #ty_generics #where_clause {
      type Error = walrs_validation::FieldsetViolations;

      fn try_from(data: walrs_form::FormData) -> Result<Self, Self::Error> {
        let mut violations = walrs_validation::FieldsetViolations::new();

        #(#field_extractions)*

        if !violations.is_empty() {
          return Err(violations);
        }

        Ok(Self {
          #(#field_names),*
        })
      }
    }
  }
}

/// Helper to generate numeric extraction code.
fn gen_numeric_extraction(
  field_name: &Ident,
  field_name_str: &str,
  num_type: &Ident,
  is_option: bool,
) -> TokenStream {
  let num_type_str = num_type.to_string();
  let default_value = if is_option {
    quote! { None }
  } else {
    match num_type_str.as_str() {
      "f32" | "f64" => quote! { 0.0 },
      _ => quote! { 0 },
    }
  };

  match num_type_str.as_str() {
    "i128" => {
      if is_option {
        quote! {
          let #field_name = match data.get_direct(#field_name_str) {
            Some(walrs_validation::Value::Str(s)) => {
              match s.parse::<i128>() {
                Ok(v) => Some(v),
                Err(_) => {
                  violations.add(
                    #field_name_str,
                    walrs_validation::Violation::new(
                      walrs_validation::ViolationType::TypeMismatch,
                      concat!("Expected valid ", #num_type_str, " string")
                    )
                  );
                  None
                }
              }
            }
            Some(walrs_validation::Value::I64(n)) => Some(*n as i128),
            Some(walrs_validation::Value::U64(n)) => Some(*n as i128),
            Some(walrs_validation::Value::Null) | None => None,
            Some(_) => {
              violations.add(
                #field_name_str,
                walrs_validation::Violation::new(
                  walrs_validation::ViolationType::TypeMismatch,
                  concat!("Expected ", #num_type_str, " or null")
                )
              );
              None
            }
          };
        }
      } else {
        quote! {
          let #field_name = match data.get_direct(#field_name_str) {
            Some(walrs_validation::Value::Str(s)) => {
              match s.parse::<i128>() {
                Ok(v) => v,
                Err(_) => {
                  violations.add(
                    #field_name_str,
                    walrs_validation::Violation::new(
                      walrs_validation::ViolationType::TypeMismatch,
                      concat!("Expected valid ", #num_type_str, " string")
                    )
                  );
                  #default_value
                }
              }
            }
            Some(walrs_validation::Value::I64(n)) => *n as i128,
            Some(walrs_validation::Value::U64(n)) => *n as i128,
            Some(_) => {
              violations.add(
                #field_name_str,
                walrs_validation::Violation::new(
                  walrs_validation::ViolationType::TypeMismatch,
                  concat!("Expected ", #num_type_str)
                )
              );
              #default_value
            }
            None => {
              violations.add(#field_name_str, walrs_validation::Violation::value_missing());
              #default_value
            }
          };
        }
      }
    }
    "i8" | "i16" | "i32" | "i64" | "isize" => {
      if is_option {
        quote! {
          let #field_name = match data.get_direct(#field_name_str) {
            Some(walrs_validation::Value::I64(n)) => {
              match #num_type::try_from(*n) {
                Ok(v) => Some(v),
                Err(_) => {
                  violations.add(
                    #field_name_str,
                    walrs_validation::Violation::new(
                      walrs_validation::ViolationType::TypeMismatch,
                      concat!("Value out of range for ", #num_type_str)
                    )
                  );
                  None
                }
              }
            }
            Some(walrs_validation::Value::U64(n)) => {
              match #num_type::try_from(*n) {
                Ok(v) => Some(v),
                Err(_) => {
                  violations.add(
                    #field_name_str,
                    walrs_validation::Violation::new(
                      walrs_validation::ViolationType::TypeMismatch,
                      concat!("Value out of range for ", #num_type_str)
                    )
                  );
                  None
                }
              }
            }
            Some(walrs_validation::Value::Null) | None => None,
            Some(_) => {
              violations.add(
                #field_name_str,
                walrs_validation::Violation::new(
                  walrs_validation::ViolationType::TypeMismatch,
                  concat!("Expected ", #num_type_str, " or null")
                )
              );
              None
            }
          };
        }
      } else {
        quote! {
          let #field_name = match data.get_direct(#field_name_str) {
            Some(walrs_validation::Value::I64(n)) => {
              match #num_type::try_from(*n) {
                Ok(v) => v,
                Err(_) => {
                  violations.add(
                    #field_name_str,
                    walrs_validation::Violation::new(
                      walrs_validation::ViolationType::TypeMismatch,
                      concat!("Value out of range for ", #num_type_str)
                    )
                  );
                  #default_value
                }
              }
            }
            Some(walrs_validation::Value::U64(n)) => {
              match #num_type::try_from(*n) {
                Ok(v) => v,
                Err(_) => {
                  violations.add(
                    #field_name_str,
                    walrs_validation::Violation::new(
                      walrs_validation::ViolationType::TypeMismatch,
                      concat!("Value out of range for ", #num_type_str)
                    )
                  );
                  #default_value
                }
              }
            }
            Some(_) => {
              violations.add(
                #field_name_str,
                walrs_validation::Violation::new(
                  walrs_validation::ViolationType::TypeMismatch,
                  concat!("Expected ", #num_type_str)
                )
              );
              #default_value
            }
            None => {
              violations.add(#field_name_str, walrs_validation::Violation::value_missing());
              #default_value
            }
          };
        }
      }
    }
    "u128" => {
      if is_option {
        quote! {
          let #field_name = match data.get_direct(#field_name_str) {
            Some(walrs_validation::Value::Str(s)) => {
              match s.parse::<u128>() {
                Ok(v) => Some(v),
                Err(_) => {
                  violations.add(
                    #field_name_str,
                    walrs_validation::Violation::new(
                      walrs_validation::ViolationType::TypeMismatch,
                      concat!("Expected valid ", #num_type_str, " string")
                    )
                  );
                  None
                }
              }
            }
            Some(walrs_validation::Value::U64(n)) => Some(*n as u128),
            Some(walrs_validation::Value::I64(n)) if *n >= 0 => Some(*n as u128),
            Some(walrs_validation::Value::Null) | None => None,
            Some(_) => {
              violations.add(
                #field_name_str,
                walrs_validation::Violation::new(
                  walrs_validation::ViolationType::TypeMismatch,
                  concat!("Expected ", #num_type_str, " or null")
                )
              );
              None
            }
          };
        }
      } else {
        quote! {
          let #field_name = match data.get_direct(#field_name_str) {
            Some(walrs_validation::Value::Str(s)) => {
              match s.parse::<u128>() {
                Ok(v) => v,
                Err(_) => {
                  violations.add(
                    #field_name_str,
                    walrs_validation::Violation::new(
                      walrs_validation::ViolationType::TypeMismatch,
                      concat!("Expected valid ", #num_type_str, " string")
                    )
                  );
                  #default_value
                }
              }
            }
            Some(walrs_validation::Value::U64(n)) => *n as u128,
            Some(walrs_validation::Value::I64(n)) if *n >= 0 => *n as u128,
            Some(_) => {
              violations.add(
                #field_name_str,
                walrs_validation::Violation::new(
                  walrs_validation::ViolationType::TypeMismatch,
                  concat!("Expected ", #num_type_str)
                )
              );
              #default_value
            }
            None => {
              violations.add(#field_name_str, walrs_validation::Violation::value_missing());
              #default_value
            }
          };
        }
      }
    }
    "u8" | "u16" | "u32" | "u64" | "usize" => {
      if is_option {
        quote! {
          let #field_name = match data.get_direct(#field_name_str) {
            Some(walrs_validation::Value::U64(n)) => {
              match #num_type::try_from(*n) {
                Ok(v) => Some(v),
                Err(_) => {
                  violations.add(
                    #field_name_str,
                    walrs_validation::Violation::new(
                      walrs_validation::ViolationType::TypeMismatch,
                      concat!("Value out of range for ", #num_type_str)
                    )
                  );
                  None
                }
              }
            }
            Some(walrs_validation::Value::I64(n)) if *n >= 0 => {
              match #num_type::try_from(*n) {
                Ok(v) => Some(v),
                Err(_) => {
                  violations.add(
                    #field_name_str,
                    walrs_validation::Violation::new(
                      walrs_validation::ViolationType::TypeMismatch,
                      concat!("Value out of range for ", #num_type_str)
                    )
                  );
                  None
                }
              }
            }
            Some(walrs_validation::Value::Null) | None => None,
            Some(_) => {
              violations.add(
                #field_name_str,
                walrs_validation::Violation::new(
                  walrs_validation::ViolationType::TypeMismatch,
                  concat!("Expected ", #num_type_str, " or null")
                )
              );
              None
            }
          };
        }
      } else {
        quote! {
          let #field_name = match data.get_direct(#field_name_str) {
            Some(walrs_validation::Value::U64(n)) => {
              match #num_type::try_from(*n) {
                Ok(v) => v,
                Err(_) => {
                  violations.add(
                    #field_name_str,
                    walrs_validation::Violation::new(
                      walrs_validation::ViolationType::TypeMismatch,
                      concat!("Value out of range for ", #num_type_str)
                    )
                  );
                  #default_value
                }
              }
            }
            Some(walrs_validation::Value::I64(n)) if *n >= 0 => {
              match #num_type::try_from(*n) {
                Ok(v) => v,
                Err(_) => {
                  violations.add(
                    #field_name_str,
                    walrs_validation::Violation::new(
                      walrs_validation::ViolationType::TypeMismatch,
                      concat!("Value out of range for ", #num_type_str)
                    )
                  );
                  #default_value
                }
              }
            }
            Some(_) => {
              violations.add(
                #field_name_str,
                walrs_validation::Violation::new(
                  walrs_validation::ViolationType::TypeMismatch,
                  concat!("Expected ", #num_type_str)
                )
              );
              #default_value
            }
            None => {
              violations.add(#field_name_str, walrs_validation::Violation::value_missing());
              #default_value
            }
          };
        }
      }
    }
    "f32" | "f64" => {
      if is_option {
        quote! {
          let #field_name = match data.get_direct(#field_name_str) {
            Some(walrs_validation::Value::F64(n)) => Some(*n as #num_type),
            Some(walrs_validation::Value::I64(n)) => Some(*n as #num_type),
            Some(walrs_validation::Value::U64(n)) => Some(*n as #num_type),
            Some(walrs_validation::Value::Null) | None => None,
            Some(_) => {
              violations.add(
                #field_name_str,
                walrs_validation::Violation::new(
                  walrs_validation::ViolationType::TypeMismatch,
                  concat!("Expected ", #num_type_str, " or null")
                )
              );
              None
            }
          };
        }
      } else {
        quote! {
          let #field_name = match data.get_direct(#field_name_str) {
            Some(walrs_validation::Value::F64(n)) => *n as #num_type,
            Some(walrs_validation::Value::I64(n)) => *n as #num_type,
            Some(walrs_validation::Value::U64(n)) => *n as #num_type,
            Some(_) => {
              violations.add(
                #field_name_str,
                walrs_validation::Violation::new(
                  walrs_validation::ViolationType::TypeMismatch,
                  concat!("Expected ", #num_type_str)
                )
              );
              #default_value
            }
            None => {
              violations.add(#field_name_str, walrs_validation::Violation::value_missing());
              #default_value
            }
          };
        }
      }
    }
    _ => {
      // Unknown numeric type, generate a default
      quote! {
        let #field_name = #default_value;
      }
    }
  }
}
