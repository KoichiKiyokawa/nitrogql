use std::collections::{HashMap, HashSet};

use thiserror::Error;

use crate::graphql_parser::ast::{
    base::{HasPos, Ident, Pos},
    directive::Directive,
    type_system::{
        ArgumentsDefinition, DirectiveDefinition, ObjectTypeDefinition, ScalarTypeDefinition,
        SchemaDefinition, TypeDefinition, TypeSystemDefinition,
    },
    TypeSystemDocument,
};

use self::{
    builtins::generate_builtins, check_directive_recursion::check_directive_recursion,
    definition_map::DefinitionMap, types::kind_of_type,
};

mod builtins;
mod check_directive_recursion;
mod definition_map;
mod tests;
mod types;

/// Checks for invalid type system definition document.
pub fn check_type_system_document(document: &TypeSystemDocument) -> Vec<CheckTypeSystemError> {
    let mut definition_map = generate_definition_map(document);
    let (builtin_types, builtin_directives) = generate_builtins();
    definition_map
        .types
        .extend(builtin_types.iter().map(|(key, def)| (*key, def)));
    definition_map
        .directives
        .extend(builtin_directives.iter().map(|(key, def)| (*key, def)));

    let definition_map = definition_map;

    let mut result = vec![];

    for def in document.definitions.iter() {
        match def {
            TypeSystemDefinition::SchemaDefinition(ref d) => {
                check_schema(d, &definition_map, &mut result);
            }
            TypeSystemDefinition::TypeDefinition(ref d) => match d {
                TypeDefinition::Scalar(ref d) => {
                    check_scalar(d, &definition_map, &mut result);
                }
                TypeDefinition::Object(ref d) => {
                    check_object(d, &definition_map, &mut result);
                }
                _ => {}
            },
            TypeSystemDefinition::DirectiveDefinition(ref d) => {
                check_directive(d, &definition_map, &mut result);
            }
        }
    }

    // result.append(&mut validate_scalars(
    //     &scalar_definitions[..],
    //     &directive_by_name,
    // ));

    result
}

#[derive(Error, Debug)]
pub enum CheckTypeSystemError {
    #[error("Name that starts with '__' is reserved")]
    UnscoUnsco { position: Pos },
    #[error("Name '{name}' is duplicated")]
    DuplicatedName { position: Pos, name: String },
    #[error("Directive name '{name}' is not found")]
    UnknownDirective { position: Pos, name: String },
    #[error("Directive '{name}' is not allowed for this location")]
    DirectiveLocationNotAllowed { position: Pos, name: String },
    #[error("Repeated application of directive '{name}' is not allowed")]
    RepeatedDirective { position: Pos, name: String },
    #[error("Directive '{name}' is recursing")]
    RecursingDirective { position: Pos, name: String },
    #[error("Output type '{name}' is not allowed here")]
    NoOutputType { position: Pos, name: String },
    #[error("Input type '{name}' is not allowed here")]
    NoInputType { position: Pos, name: String },
}

fn generate_definition_map<'a>(document: &'a TypeSystemDocument<'a>) -> DefinitionMap<'a> {
    let mut result = DefinitionMap::new();
    for def in document.definitions.iter() {
        match def {
            TypeSystemDefinition::SchemaDefinition(schema) => {
                result.schema.insert(schema);
            }
            TypeSystemDefinition::TypeDefinition(def) => {
                result.types.insert(
                    def.name().expect("Type definition should always have name"),
                    def,
                );
            }
            TypeSystemDefinition::DirectiveDefinition(def) => {
                result.directives.insert(def.name.name, def);
            }
        }
    }

    result
}

fn check_schema(
    d: &SchemaDefinition,
    definitions: &DefinitionMap,
    result: &mut Vec<CheckTypeSystemError>,
) {
    check_directives(definitions, &d.directives, "SCHEMA", result);
}

fn check_directive<'a>(
    d: &DirectiveDefinition,
    definitions: &DefinitionMap,
    result: &mut Vec<CheckTypeSystemError>,
) {
    check_directive_recursion(definitions, d, result);

    if name_starts_with_unscounsco(&d.name) {
        result.push(CheckTypeSystemError::UnscoUnsco {
            position: *d.name.position(),
        });
    }
    if let Some(ref arg) = d.arguments {
        check_arguments_definition(arg, definitions, result);
    }
}

fn check_scalar(
    scalar: &ScalarTypeDefinition,
    definition_map: &DefinitionMap,
    result: &mut Vec<CheckTypeSystemError>,
) {
    if name_starts_with_unscounsco(&scalar.name) {
        result.push(CheckTypeSystemError::UnscoUnsco {
            position: *scalar.name.position(),
        })
    }
    check_directives(definition_map, &scalar.directives, "SCALAR", result);
}

fn check_object(
    object: &ObjectTypeDefinition,
    definitions: &DefinitionMap,
    result: &mut Vec<CheckTypeSystemError>,
) {
    if name_starts_with_unscounsco(&object.name) {
        result.push(CheckTypeSystemError::UnscoUnsco {
            position: *object.name.position(),
        })
    }
    let mut seen_fields = vec![];
    for f in object.fields.iter() {
        if seen_fields.contains(&f.name.name) {
            result.push(CheckTypeSystemError::DuplicatedName {
                position: *f.name.position(),
                name: f.name.name.to_owned(),
            });
        } else {
            seen_fields.push(f.name.name);
        }
        if name_starts_with_unscounsco(&f.name) {
            result.push(CheckTypeSystemError::UnscoUnsco {
                position: *f.name.position(),
            })
        }
        if kind_of_type(definitions, &f.r#type).map_or(false, |k| !k.is_output_type()) {
            result.push(CheckTypeSystemError::NoInputType {
                position: *f.r#type.position(),
                name: f.r#type.unwrapped_type().name.name.to_owned(),
            });
        }
        if let Some(ref arg) = f.arguments {
            check_arguments_definition(arg, definitions, result)
        }
    }
}

fn check_arguments_definition(
    def: &ArgumentsDefinition,
    definitions: &DefinitionMap,
    result: &mut Vec<CheckTypeSystemError>,
) {
    let mut argument_names = vec![];
    for v in def.input_values.iter() {
        if name_starts_with_unscounsco(&v.name) {
            result.push(CheckTypeSystemError::UnscoUnsco {
                position: *v.name.position(),
            });
        }
        if argument_names.contains(&v.name.name) {
            result.push(CheckTypeSystemError::DuplicatedName {
                position: *v.name.position(),
                name: v.name.name.to_owned(),
            })
        } else {
            argument_names.push(v.name.name);
        }
        let type_is_not_input_type =
            kind_of_type(definitions, &v.r#type).map_or(false, |k| !k.is_input_type());
        if type_is_not_input_type {
            result.push(CheckTypeSystemError::NoOutputType {
                position: *v.r#type.position(),
                name: v.r#type.unwrapped_type().name.name.to_owned(),
            })
        }

        check_directives(definitions, &v.directives, "ARGUMENT_DEFINITION", result)
    }
}

fn name_starts_with_unscounsco(name: &Ident) -> bool {
    name.name.starts_with("__")
}

fn check_directives(
    definitions: &DefinitionMap,
    directives: &[Directive],
    current_position: &str,
    result: &mut Vec<CheckTypeSystemError>,
) {
    let mut seen_directives = vec![];
    for d in directives {
        match definitions.directives.get(d.name.name) {
            None => result.push(CheckTypeSystemError::UnknownDirective {
                position: *d.name.position(),
                name: d.name.name.to_owned(),
            }),
            Some(def) => {
                if def
                    .locations
                    .iter()
                    .find(|loc| loc.name == current_position)
                    .is_none()
                {
                    result.push(CheckTypeSystemError::DirectiveLocationNotAllowed {
                        position: *d.position(),
                        name: d.name.name.to_owned(),
                    });
                }
                if seen_directives.contains(&d.name.name) {
                    if def.repeatable.is_none() {
                        result.push(CheckTypeSystemError::RepeatedDirective {
                            position: *d.position(),
                            name: d.name.name.to_owned(),
                        })
                    }
                } else {
                    seen_directives.push(d.name.name);
                }
            }
        }
    }
}