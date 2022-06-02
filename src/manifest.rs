// Responsible for emitting Cargo.tomls

use std::collections::HashMap;

use serde::Serialize;

use crate::directives::RustEdition;

#[derive(Serialize)]
pub struct CargoTomlPackage {
    pub name: String,
    pub version: String,
    pub edition: RustEdition,
}

#[derive(Serialize)]
pub struct CargoTomlDependencyObj {
    pub version: String,
    pub features: Vec<String>,
}

#[derive(Serialize)]
pub struct CargoToml {
    pub package: CargoTomlPackage,
    pub dependencies: HashMap<String, CargoTomlDependencyObj>,
}