use chumsky::{prelude::*, error::Simple};

use crate::{directives::{Directive, Dependency, RustEdition, OptimisationType}, crates::{CrateVersion, Feature}};

fn semver() -> impl Parser<char, semver::Version, Error=Simple<char>> {
    filter(|x: &char| x.is_alphanumeric() || x == &'-' || x == &'_' || x == &'+' || x == &'.')
        .repeated()
        .try_map(|x, span| {
            let a = String::from_iter(x);
            semver::Version::parse(&a)
                .map_err(|e|
                    Simple::custom(span, format!("Invalid SemVer: {e}")))
        })
}

pub fn directive() -> impl Parser<char, Directive, Error=Simple<char>> {
    recursive(|_directive| {
        let name =
            filter(|x: &char| x.is_alphanumeric() || x == &'-' || x == &'_')
            .repeated()
            .map(String::from_iter);
        
        let loose_name =
            filter(|x: &char| x.is_alphanumeric() || x == &'-' || x == &'_' || x == &'+' || x == &'.')
            .repeated()
            .map(String::from_iter);
        
        let crate_version = choice((
            text::keyword("latest").map(|_| CrateVersion::Latest),
            semver().map(CrateVersion::Specific)
        ));

        let feature = choice((
            just('!').ignore_then(name).map(Feature::Disable),
            name.map(Feature::Enable)
        ));

        let crate_spec = name.padded()
            .then(crate_version.padded())
            .then(
                text::keyword("with").padded()
                .ignore_then(
                    feature
                    .separated_by(filter(|x: &char| x.is_whitespace()).repeated().at_least(1))
                    .delimited_by(just('(').padded(), just(')').padded())
                )
                .or_not()
            )
            .map(|((crt, ver), feats): ((_, _), Option<Vec<Feature>>)| {
                Dependency {
                    name: crt,
                    version: ver,
                    features: feats.unwrap_or(vec![])
                }
            });

        let dep = text::keyword("use")
            .ignore_then(
                crate_spec
                    .separated_by(just(',').padded())
                    .at_least(1)
            )
            .map(Directive::Dependency);

        let edition = text::keyword("edition")
            .padded()
            .ignore_then(
                name
                .try_map(|x, span|
                    RustEdition::try_from(x.as_str())
                        .map_err(|_| Simple::custom(span, "Invalid Rust edition"))
                )
            )
            .map(Directive::Edition);

        let toolchain = text::keyword("toolchain")
            .padded()
            .ignore_then(loose_name)
            .then(
                text::keyword("targeting")
                .padded()
                .ignore_then(loose_name)
                .or_not()
            )
            .map(|(chain, tgt)|
                match tgt {
                    Some(x) => Directive::ToolchainWithTarget(chain, x),
                    None => Directive::Toolchain(chain),
                });

        let optimise = choice((
            text::keyword("optimise"),
            text::keyword("optimize")
        ))
            .padded()
            .ignore_then(
                name
                .try_map(|x, span|
                    OptimisationType::try_from(x.as_str())
                        .map_err(|_| Simple::custom(span, "Invalid optimisation type"))
                )
            )
            .map(Directive::Optimise);
        
        choice((
            dep,
            edition,
            toolchain,
            optimise
        ))
    })
}