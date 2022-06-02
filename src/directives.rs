use crate::crates::{CrateVersion, Feature};

#[derive(Clone, Debug, Serialize)]
pub enum RustEdition {
    #[serde(rename="2015")]
    Rust2015,
    #[serde(rename="2018")]
    Rust2018,
    #[serde(rename="2021")]
    Rust2021
}

#[derive(Clone, Debug)]
pub enum OptimisationType {
    Debug,
    Release
}

impl TryFrom<&str> for OptimisationType {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "debug" => Ok(Self::Debug),
            "release" => Ok(Self::Release),
            "nyoom" => Ok(Self::Release),
            _ => Err(())
        }
    }    
}

impl OptimisationType {
    pub fn directory(&self) -> String {
        match self {
            OptimisationType::Debug => "debug",
            OptimisationType::Release => "release",
        }.into()
    }
}

impl TryFrom<&str> for RustEdition {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "2015" => Ok(Self::Rust2015),
            "2018" => Ok(Self::Rust2018),
            "2021" => Ok(Self::Rust2021),
            _ => Err(())
        }
    }
}

#[derive(Debug, Clone)]
pub struct Dependency {
    pub name: String,
    pub version: CrateVersion,
    pub features: Vec<Feature>,
}

impl Dependency {
    pub async fn get_latest_version(&self) -> String {
        crate::crates::get_latest_version(&self.name).await
    }
}

#[derive(Debug)]
pub enum Directive {
    Dependency(Vec<Dependency>),
    Toolchain(String),
    ToolchainWithTarget(String, String), // toolchain + target
    When(String, Box<Directive>), // condition
    Edition(RustEdition),
    Optimise(OptimisationType),
}