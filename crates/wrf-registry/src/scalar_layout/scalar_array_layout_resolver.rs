use std::collections::HashMap;

use crate::{
    DefinitionParameterIndex, RegistryDocument, RegistryResolutionError,
    RegistryResolutionErrorKind, RegistryResolutionResult, RegistryValueType,
    ResolvedScalarArrayLayout, ResolvedScalarArrayMember, RuntimeConfigurationChoice,
    RustDenseScalarIndex, SourceLocation, WrfPackedScalarIndex,
};

pub(crate) struct ScalarArrayLayoutResolver;

#[derive(Clone)]
struct ScalarArrayDefinitionMember {
    location: SourceLocation,
    name: String,
}

struct ScalarArrayDefinition {
    location: SourceLocation,
    name: String,
    members: Vec<ScalarArrayDefinitionMember>,
    member_index_by_name: HashMap<String, usize>,
}

impl ScalarArrayLayoutResolver {
    pub(crate) fn resolve(
        document: &RegistryDocument,
        choices: &[RuntimeConfigurationChoice],
    ) -> RegistryResolutionResult<Vec<ResolvedScalarArrayLayout>> {
        let document_location = SourceLocation::new(&document.source_name, 1, 1);
        let configurations_by_name = Self::validate_runtime_configurations(document)?;
        let choices_by_name =
            Self::validate_choices(choices, &configurations_by_name, &document_location)?;
        let definitions = Self::collect_scalar_array_definitions(document)?;
        let definition_index_by_name: HashMap<_, _> = definitions
            .iter()
            .enumerate()
            .map(|(index, definition)| (definition.name.as_str(), index))
            .collect();
        Self::validate_packages(
            document,
            &configurations_by_name,
            &definitions,
            &definition_index_by_name,
        )?;
        let mut layouts: Vec<_> = definitions
            .iter()
            .map(|definition| ResolvedScalarArrayLayout {
                location: definition.location.clone(),
                scalar_array_name: definition.name.clone(),
                definition_member_count: definition.members.len(),
                members: Vec::new(),
            })
            .collect();

        for package in document.packages() {
            let condition = package.condition();
            if choices_by_name.get(condition.configuration_name()) != Some(&condition.choice()) {
                continue;
            }

            for group in package.variable_groups() {
                let Some(&definition_index) =
                    definition_index_by_name.get(group.scalar_array_name())
                else {
                    return Err(RegistryResolutionError::new(
                        group.location().clone(),
                        RegistryResolutionErrorKind::UnknownScalarArray {
                            scalar_array_name: group.scalar_array_name().to_owned(),
                        },
                    ));
                };
                let definition = &definitions[definition_index];
                let layout = &mut layouts[definition_index];
                for member_name in group.members() {
                    let Some(&parameter_index) = definition.member_index_by_name.get(member_name)
                    else {
                        return Err(RegistryResolutionError::new(
                            group.location().clone(),
                            RegistryResolutionErrorKind::UnknownScalarArrayMember {
                                scalar_array_name: group.scalar_array_name().to_owned(),
                                member_name: member_name.clone(),
                            },
                        ));
                    };
                    if layout
                        .members
                        .iter()
                        .any(|active_member| active_member.name() == member_name)
                    {
                        continue;
                    }

                    let dense_index = layout.members.len();
                    let definition_member = &definition.members[parameter_index];
                    layout.members.push(ResolvedScalarArrayMember {
                        location: definition_member.location.clone(),
                        name: member_name.clone(),
                        definition_parameter_index: DefinitionParameterIndex::new(parameter_index),
                        wrf_packed_scalar_index: WrfPackedScalarIndex::new(dense_index + 2),
                        rust_dense_scalar_index: RustDenseScalarIndex::new(dense_index),
                    });
                }
            }
        }

        Ok(layouts)
    }

    fn validate_packages(
        document: &RegistryDocument,
        configurations_by_name: &HashMap<&str, &crate::RuntimeConfiguration>,
        definitions: &[ScalarArrayDefinition],
        definition_index_by_name: &HashMap<&str, usize>,
    ) -> RegistryResolutionResult<()> {
        for package in document.packages() {
            let condition = package.condition();
            let configuration = configurations_by_name
                .get(condition.configuration_name())
                .ok_or_else(|| {
                    RegistryResolutionError::new(
                        condition.location().clone(),
                        RegistryResolutionErrorKind::UnknownPackageConditionConfiguration {
                            name: condition.configuration_name().to_owned(),
                        },
                    )
                })?;
            if configuration.value_type() != &RegistryValueType::Integer {
                return Err(RegistryResolutionError::new(
                    condition.location().clone(),
                    RegistryResolutionErrorKind::NonIntegerPackageConditionConfiguration {
                        name: condition.configuration_name().to_owned(),
                    },
                ));
            }

            for group in package.variable_groups() {
                let Some(&definition_index) =
                    definition_index_by_name.get(group.scalar_array_name())
                else {
                    return Err(RegistryResolutionError::new(
                        group.location().clone(),
                        RegistryResolutionErrorKind::UnknownScalarArray {
                            scalar_array_name: group.scalar_array_name().to_owned(),
                        },
                    ));
                };
                let definition = &definitions[definition_index];
                for member_name in group.members() {
                    if member_name == "-" {
                        return Err(RegistryResolutionError::new(
                            group.location().clone(),
                            RegistryResolutionErrorKind::ReservedScalarArrayMemberActivation {
                                scalar_array_name: group.scalar_array_name().to_owned(),
                            },
                        ));
                    }
                    if !definition.member_index_by_name.contains_key(member_name) {
                        return Err(RegistryResolutionError::new(
                            group.location().clone(),
                            RegistryResolutionErrorKind::UnknownScalarArrayMember {
                                scalar_array_name: group.scalar_array_name().to_owned(),
                                member_name: member_name.clone(),
                            },
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn validate_runtime_configurations(
        document: &RegistryDocument,
    ) -> RegistryResolutionResult<HashMap<&str, &crate::RuntimeConfiguration>> {
        let mut configurations_by_name = HashMap::new();
        for configuration in document.runtime_configurations() {
            if configurations_by_name
                .insert(configuration.name(), configuration)
                .is_some()
            {
                return Err(RegistryResolutionError::new(
                    configuration.location().clone(),
                    RegistryResolutionErrorKind::DuplicateRuntimeConfiguration {
                        name: configuration.name().to_owned(),
                    },
                ));
            }
        }
        Ok(configurations_by_name)
    }

    fn validate_choices<'a>(
        choices: &'a [RuntimeConfigurationChoice],
        configurations_by_name: &HashMap<&str, &crate::RuntimeConfiguration>,
        document_location: &SourceLocation,
    ) -> RegistryResolutionResult<HashMap<&'a str, i32>> {
        let mut choices_by_name = HashMap::new();
        for choice in choices {
            if !configurations_by_name.contains_key(choice.name()) {
                return Err(RegistryResolutionError::new(
                    document_location.clone(),
                    RegistryResolutionErrorKind::UnknownRuntimeConfigurationChoice {
                        name: choice.name().to_owned(),
                    },
                ));
            }
            if choices_by_name
                .insert(choice.name(), choice.value())
                .is_some()
            {
                return Err(RegistryResolutionError::new(
                    document_location.clone(),
                    RegistryResolutionErrorKind::DuplicateRuntimeConfigurationChoice {
                        name: choice.name().to_owned(),
                    },
                ));
            }
        }
        Ok(choices_by_name)
    }

    fn collect_scalar_array_definitions(
        document: &RegistryDocument,
    ) -> RegistryResolutionResult<Vec<ScalarArrayDefinition>> {
        let mut definitions = Vec::<ScalarArrayDefinition>::new();
        let mut definition_index_by_name = HashMap::<String, usize>::new();

        for state in document
            .state_variables()
            .filter(|state| state.dimensions().is_scalar_array_member())
        {
            let Some(scalar_array_name) = state.use_association() else {
                return Err(RegistryResolutionError::new(
                    state.location().clone(),
                    RegistryResolutionErrorKind::MissingScalarArrayAssociation {
                        member_name: state.name().to_owned(),
                    },
                ));
            };
            let definition_index = match definition_index_by_name.get(scalar_array_name) {
                Some(&index) => index,
                None => {
                    let index = definitions.len();
                    definition_index_by_name.insert(scalar_array_name.to_owned(), index);
                    definitions.push(ScalarArrayDefinition {
                        location: state.location().clone(),
                        name: scalar_array_name.to_owned(),
                        members: Vec::new(),
                        member_index_by_name: HashMap::new(),
                    });
                    index
                }
            };
            let definition = &mut definitions[definition_index];
            let parameter_index = definition.members.len();
            if definition
                .member_index_by_name
                .insert(state.name().to_owned(), parameter_index)
                .is_some()
            {
                return Err(RegistryResolutionError::new(
                    state.location().clone(),
                    RegistryResolutionErrorKind::DuplicateScalarArrayDefinitionMember {
                        scalar_array_name: scalar_array_name.to_owned(),
                        member_name: state.name().to_owned(),
                    },
                ));
            }
            definition.members.push(ScalarArrayDefinitionMember {
                location: state.location().clone(),
                name: state.name().to_owned(),
            });
        }

        for definition in &definitions {
            if definition
                .members
                .first()
                .map(|member| member.name.as_str())
                != Some("-")
            {
                return Err(RegistryResolutionError::new(
                    definition.location.clone(),
                    RegistryResolutionErrorKind::MissingReservedScalarArrayMember {
                        scalar_array_name: definition.name.clone(),
                    },
                ));
            }
        }
        Ok(definitions)
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use crate::{RegistryParser, RegistryResolutionErrorKind, RuntimeConfigurationChoice};

    const FIXTURE: &str = "\
dimspec i 1 standard_domain x west_east
dimspec k 2 standard_domain z bottom_top
dimspec j 3 standard_domain y south_north
rconfig integer mp_physics namelist,physics max_domains -1 irh mp_physics \"\" \"\"
state real - ikjftb moist 1 - - - - -
state real qv ikjftb moist 1 - - QVAPOR vapor 1
state real qc ikjftb moist 1 - - QCLOUD cloud 1
state real qr ikjftb moist 1 - - QRAIN rain 1
package passiveqv mp_physics==0 - moist:qv
package kesslerscheme mp_physics==1 - moist:qv,qc,qr
package reordered mp_physics==2 - moist:qr,qv,qc
package repeated_a mp_physics==4 - moist:qv,qc
package repeated_b mp_physics==4 - moist:qv,qr
";

    fn resolve(choice: i32) -> Vec<crate::ResolvedScalarArrayLayout> {
        let document = RegistryParser::parse("Registry.fixture", FIXTURE).unwrap();
        document
            .resolve_scalar_array_layouts(&[RuntimeConfigurationChoice::new("mp_physics", choice)])
            .unwrap()
    }

    #[test]
    fn resolves_canonical_definition_packed_and_dense_indices() {
        let layouts = resolve(1);
        let moist = &layouts[0];

        assert_eq!(moist.definition_member_count(), 4);
        assert_eq!(moist.reserved_parameter_index().as_usize(), 0);
        assert_eq!(moist.reserved_packed_scalar_index().as_usize(), 1);
        let projection: Vec<_> = moist
            .members()
            .iter()
            .map(|member| {
                (
                    member.name(),
                    member.definition_parameter_index().as_usize(),
                    member.wrf_packed_scalar_index().as_usize(),
                    member.rust_dense_scalar_index().as_usize(),
                )
            })
            .collect();
        assert_eq!(
            projection,
            [("qv", 1, 2, 0), ("qc", 2, 3, 1), ("qr", 3, 4, 2)]
        );
    }

    #[test]
    fn preserves_reordered_activation_and_definition_indices() {
        let layouts = resolve(2);
        let projection: Vec<_> = layouts[0]
            .members()
            .iter()
            .map(|member| {
                (
                    member.name(),
                    member.definition_parameter_index().as_usize(),
                    member.wrf_packed_scalar_index().as_usize(),
                )
            })
            .collect();

        assert_eq!(projection, [("qr", 3, 2), ("qv", 1, 3), ("qc", 2, 4)]);
    }

    #[test]
    fn deduplicates_repeated_activation_in_package_source_order() {
        let layouts = resolve(4);
        let names: Vec<_> = layouts[0]
            .members()
            .iter()
            .map(crate::ResolvedScalarArrayMember::name)
            .collect();

        assert_eq!(names, ["qv", "qc", "qr"]);
        assert_eq!(
            layouts[0].members()[2].wrf_packed_scalar_index().as_usize(),
            4
        );
    }

    #[test]
    fn inactive_selection_retains_only_the_reserved_layout_contract() {
        let layouts = resolve(-9);

        assert_eq!(layouts.len(), 1);
        assert!(layouts[0].members().is_empty());
        assert_eq!(layouts[0].reserved_packed_scalar_index().as_usize(), 1);
    }

    #[test]
    fn rejects_unknown_array_and_member_without_partial_layout() {
        for group in ["other:qv", "moist:unknown"] {
            let source = format!("{FIXTURE}package invalid mp_physics==7 - {group}\n");
            let document = RegistryParser::parse("Registry.invalid", &source).unwrap();
            let error = document
                .resolve_scalar_array_layouts(&[RuntimeConfigurationChoice::new("mp_physics", 7)])
                .unwrap_err();

            assert_eq!(error.location().line(), 14);
        }
    }

    #[test]
    fn validates_inactive_package_references_and_rejects_reserved_activation() {
        for (group, expected_kind) in [
            (
                "other:qv",
                RegistryResolutionErrorKind::UnknownScalarArray {
                    scalar_array_name: "other".to_owned(),
                },
            ),
            (
                "moist:unknown",
                RegistryResolutionErrorKind::UnknownScalarArrayMember {
                    scalar_array_name: "moist".to_owned(),
                    member_name: "unknown".to_owned(),
                },
            ),
            (
                "moist:-",
                RegistryResolutionErrorKind::ReservedScalarArrayMemberActivation {
                    scalar_array_name: "moist".to_owned(),
                },
            ),
        ] {
            let source = format!("{FIXTURE}package invalid mp_physics==7 - {group}\n");
            let document = RegistryParser::parse("Registry.inactive-invalid", &source).unwrap();
            let error = document
                .resolve_scalar_array_layouts(&[RuntimeConfigurationChoice::new("mp_physics", -9)])
                .unwrap_err();

            assert_eq!(error.kind(), &expected_kind);
        }
    }

    #[test]
    fn repeated_and_concurrent_resolution_is_deterministic() {
        let document = RegistryParser::parse("Registry.fixture", FIXTURE).unwrap();
        let expected = document
            .resolve_scalar_array_layouts(&[RuntimeConfigurationChoice::new("mp_physics", 2)])
            .unwrap();
        let handles: Vec<_> = (0..8)
            .map(|_| {
                thread::spawn(move || {
                    let document = RegistryParser::parse("Registry.fixture", FIXTURE).unwrap();
                    document
                        .resolve_scalar_array_layouts(&[RuntimeConfigurationChoice::new(
                            "mp_physics",
                            2,
                        )])
                        .unwrap()
                })
            })
            .collect();

        for handle in handles {
            assert_eq!(handle.join().unwrap(), expected);
        }
    }

    #[test]
    fn rejects_duplicate_and_unknown_runtime_configuration_choices() {
        let document = RegistryParser::parse("Registry.fixture", FIXTURE).unwrap();
        let duplicate = document
            .resolve_scalar_array_layouts(&[
                RuntimeConfigurationChoice::new("mp_physics", 1),
                RuntimeConfigurationChoice::new("mp_physics", 2),
            ])
            .unwrap_err();
        assert_eq!(
            duplicate.kind(),
            &RegistryResolutionErrorKind::DuplicateRuntimeConfigurationChoice {
                name: "mp_physics".to_owned()
            }
        );

        let unknown = document
            .resolve_scalar_array_layouts(&[RuntimeConfigurationChoice::new("unknown", 1)])
            .unwrap_err();
        assert_eq!(
            unknown.kind(),
            &RegistryResolutionErrorKind::UnknownRuntimeConfigurationChoice {
                name: "unknown".to_owned()
            }
        );
    }

    #[test]
    fn rejects_unknown_noninteger_and_duplicate_condition_configurations() {
        let sources = [
            (
                "package p missing==1\n",
                RegistryResolutionErrorKind::UnknownPackageConditionConfiguration {
                    name: "missing".to_owned(),
                },
            ),
            (
                "rconfig real option namelist,test 1 0 - option \"\" \"\"\npackage p option==1\n",
                RegistryResolutionErrorKind::NonIntegerPackageConditionConfiguration {
                    name: "option".to_owned(),
                },
            ),
            (
                "rconfig integer option namelist,test 1 0 - option \"\" \"\"\nrconfig integer option namelist,test 1 0 - option \"\" \"\"\npackage p option==1\n",
                RegistryResolutionErrorKind::DuplicateRuntimeConfiguration {
                    name: "option".to_owned(),
                },
            ),
        ];

        for (source, expected_kind) in sources {
            let document = RegistryParser::parse("Registry.configuration", source).unwrap();
            let error = document.resolve_scalar_array_layouts(&[]).unwrap_err();
            assert_eq!(error.kind(), &expected_kind);
        }
    }

    #[test]
    fn rejects_missing_reserved_duplicate_and_unassociated_scalar_definitions() {
        let prefix = "dimspec i 1 standard_domain x west_east\n";
        let sources = [
            (
                format!("{prefix}state real qv if moist 1 - - QV vapor 1\n"),
                RegistryResolutionErrorKind::MissingReservedScalarArrayMember {
                    scalar_array_name: "moist".to_owned(),
                },
            ),
            (
                format!(
                    "{prefix}state real - if moist 1 - - - - -\nstate real qv if moist 1 - - QV vapor 1\nstate real qv if moist 1 - - QV vapor 1\n"
                ),
                RegistryResolutionErrorKind::DuplicateScalarArrayDefinitionMember {
                    scalar_array_name: "moist".to_owned(),
                    member_name: "qv".to_owned(),
                },
            ),
            (
                format!("{prefix}state real qv if - 1 - - QV vapor 1\n"),
                RegistryResolutionErrorKind::MissingScalarArrayAssociation {
                    member_name: "qv".to_owned(),
                },
            ),
        ];

        for (source, expected_kind) in sources {
            let document = RegistryParser::parse("Registry.scalar", &source).unwrap();
            let error = document.resolve_scalar_array_layouts(&[]).unwrap_err();
            assert_eq!(error.kind(), &expected_kind);
        }
    }
}
