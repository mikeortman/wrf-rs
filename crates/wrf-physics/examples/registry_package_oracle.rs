//! Direct semantic projection of Registry package and scalar-layout behavior.

use std::env;
use std::error::Error;
use std::io;
use std::path::Path;

use wrf_physics::{MoistureSpecies, MoistureSpeciesPackage};
use wrf_registry::{
    RegistryDefinitions, RegistryDocument, RegistryParser, ResolvedScalarArrayLayout,
    RuntimeConfigurationChoice,
};

const CASE_CHOICES: [i32; 9] = [-9, -5, -4, -3, 0, 1, 2, 4, 5];

fn main() -> Result<(), Box<dyn Error>> {
    let fixture_path = env::args_os().nth(1).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: registry_package_oracle REGISTRY_FIXTURE",
        )
    })?;
    let document =
        RegistryParser::parse_file(Path::new(&fixture_path), &RegistryDefinitions::new())?;

    print_packages(&document);
    print_scalar_definition(&document)?;
    for choice in CASE_CHOICES {
        print_runtime_case(&document, choice)?;
    }

    Ok(())
}

fn print_packages(document: &RegistryDocument) {
    for package in document.packages() {
        let groups = if package.variable_groups().is_empty() {
            "-".to_owned()
        } else {
            package
                .variable_groups()
                .iter()
                .map(|group| {
                    format!(
                        "{}:{}",
                        group.scalar_array_name(),
                        group.members().join(",")
                    )
                })
                .collect::<Vec<_>>()
                .join(";")
        };
        println!(
            "PACKAGE|name={}|configuration={}|choice={}|groups={groups}",
            package.name(),
            package.condition().configuration_name(),
            package.condition().choice(),
        );
    }
}

fn print_scalar_definition(document: &RegistryDocument) -> Result<(), Box<dyn Error>> {
    let layouts = resolve_layouts(document, 1)?;
    let layout = require_moist_layout(&layouts)?;
    let first_packed = layout
        .members()
        .first()
        .ok_or_else(|| io::Error::other("canonical Kessler layout is empty"))?
        .wrf_packed_scalar_index()
        .as_usize();
    println!(
        "ARRAY|name={}|definition_member_count={}|reserved_parameter={}|first_packed={first_packed}",
        layout.scalar_array_name(),
        layout.definition_member_count(),
        layout.reserved_parameter_index().as_usize(),
    );

    let mut members = layout.members().iter().collect::<Vec<_>>();
    members.sort_by_key(|member| member.definition_parameter_index());
    for member in members {
        println!(
            "MEMBER|array={}|name={}|parameter={}",
            layout.scalar_array_name(),
            member.name(),
            member.definition_parameter_index().as_usize(),
        );
    }
    Ok(())
}

fn print_runtime_case(document: &RegistryDocument, choice: i32) -> Result<(), Box<dyn Error>> {
    let layouts = resolve_layouts(document, choice)?;
    let layout = layouts
        .iter()
        .find(|candidate| candidate.scalar_array_name() == "moist");
    if layouts.len() > usize::from(layout.is_some()) {
        return Err(io::Error::other("fixture resolved an unexpected scalar array").into());
    }

    let physics_package = layout
        .filter(|layout| layout.members().len() == 3)
        .map(MoistureSpeciesPackage::try_from_registry_layout)
        .transpose()?;
    let active_count = layout.map_or(0, |layout| layout.members().len());
    let qv = format_member(
        layout,
        physics_package.as_ref(),
        "qv",
        MoistureSpecies::WaterVapor,
    )?;
    let qc = format_member(
        layout,
        physics_package.as_ref(),
        "qc",
        MoistureSpecies::CloudWater,
    )?;
    let qr = format_member(
        layout,
        physics_package.as_ref(),
        "qr",
        MoistureSpecies::RainWater,
    )?;
    println!(
        "CASE|choice={choice}|num={}|qv={qv}|qc={qc}|qr={qr}",
        active_count + 1
    );
    Ok(())
}

fn resolve_layouts(
    document: &RegistryDocument,
    choice: i32,
) -> Result<Vec<ResolvedScalarArrayLayout>, Box<dyn Error>> {
    Ok(document
        .resolve_scalar_array_layouts(&[RuntimeConfigurationChoice::new("mp_physics", choice)])?)
}

fn require_moist_layout(
    layouts: &[ResolvedScalarArrayLayout],
) -> Result<&ResolvedScalarArrayLayout, Box<dyn Error>> {
    if layouts.len() != 1 || layouts[0].scalar_array_name() != "moist" {
        return Err(
            io::Error::other("canonical choice did not resolve exactly one moist layout").into(),
        );
    }
    Ok(&layouts[0])
}

fn format_member(
    layout: Option<&ResolvedScalarArrayLayout>,
    physics_package: Option<&MoistureSpeciesPackage>,
    member_name: &str,
    species: MoistureSpecies,
) -> Result<String, Box<dyn Error>> {
    let Some(member) = layout.and_then(|layout| {
        layout
            .members()
            .iter()
            .find(|member| member.name() == member_name)
    }) else {
        return Ok("1:F:-1".to_owned());
    };

    let dense_index = member.rust_dense_scalar_index().as_usize();
    if let Some(package) = physics_package {
        let physics_index = package.index_of(species).ok_or_else(|| {
            io::Error::other(format!(
                "physics package omitted active Registry member {member_name}"
            ))
        })?;
        if physics_index.as_usize() != dense_index {
            return Err(io::Error::other(format!(
                "physics and Registry dense positions differ for {member_name}"
            ))
            .into());
        }
    }

    Ok(format!(
        "{}:T:{dense_index}",
        member.wrf_packed_scalar_index().as_usize()
    ))
}
