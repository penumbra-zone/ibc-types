use core::fmt::Display;

use crate::prelude::*;

use ibc_proto::{ibc::core::connection::v1::Version as RawVersion, protobuf::Protobuf};

// TODO; move this to channel crate
//use crate::core::ics04_channel::channel::Order;
use crate::ConnectionError;

/// Stores the identifier and the features supported by a version
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Version {
    /// unique version identifier
    pub identifier: String,
    /// list of features compatible with the specified identifier
    pub features: Vec<String>,
}

impl Version {
    /// Checks whether or not the given feature is supported in this versin
    pub fn is_supported_feature(&self, feature: String) -> bool {
        self.features.contains(&feature)
    }
}

impl Protobuf<RawVersion> for Version {}

impl TryFrom<RawVersion> for Version {
    type Error = ConnectionError;
    fn try_from(value: RawVersion) -> Result<Self, Self::Error> {
        if value.identifier.trim().is_empty() {
            return Err(ConnectionError::EmptyVersions);
        }
        for feature in value.features.iter() {
            if feature.trim().is_empty() {
                return Err(ConnectionError::EmptyFeatures);
            }
        }
        Ok(Version {
            identifier: value.identifier,
            features: value.features,
        })
    }
}

impl From<Version> for RawVersion {
    fn from(value: Version) -> Self {
        Self {
            identifier: value.identifier,
            features: value.features,
        }
    }
}

impl Default for Version {
    fn default() -> Self {
        Version {
            identifier: "1".to_string(),
            features: vec!["ORDER_ORDERED".to_string(), "ORDER_UNORDERED".to_string()],
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Version {{ identifier: {}, features: {:?} }}",
            self.identifier, self.features,
        )
    }
}

impl Version {
    /// Returns the lists of supported versions
    pub fn compatible_versions() -> Vec<Version> {
        vec![Version::default()]
    }

    /// Selects a version from the intersection of locally supported and counterparty versions.
    pub fn select(
        supported_versions: &[Version],
        counterparty_versions: &[Version],
    ) -> Result<Version, ConnectionError> {
        let mut intersection: Vec<Version> = Vec::new();
        for s in supported_versions.iter() {
            for c in counterparty_versions.iter() {
                if c.identifier != s.identifier {
                    continue;
                }
                for feature in c.features.iter() {
                    if feature.trim().is_empty() {
                        return Err(ConnectionError::EmptyFeatures);
                    }
                }
                intersection.append(&mut vec![s.clone()]);
            }
        }
        intersection.sort_by(|a, b| a.identifier.cmp(&b.identifier));
        if intersection.is_empty() {
            return Err(ConnectionError::NoCommonVersion);
        }
        Ok(intersection[0].clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    use test_log::test;

    use ibc_proto::ibc::core::connection::v1::Version as RawVersion;

    use super::*;
    use crate::ConnectionError;

    fn good_versions() -> Vec<RawVersion> {
        vec![
            Version::default().into(),
            RawVersion {
                identifier: "2".to_string(),
                features: vec!["ORDER_RANDOM".to_string(), "ORDER_UNORDERED".to_string()],
            },
        ]
        .into_iter()
        .collect()
    }

    fn bad_versions_identifier() -> Vec<RawVersion> {
        vec![RawVersion {
            identifier: "".to_string(),
            features: vec!["ORDER_RANDOM".to_string(), "ORDER_UNORDERED".to_string()],
        }]
        .into_iter()
        .collect()
    }

    fn bad_versions_features() -> Vec<RawVersion> {
        vec![RawVersion {
            identifier: "2".to_string(),
            features: vec!["".to_string()],
        }]
        .into_iter()
        .collect()
    }

    fn overlapping() -> (Vec<Version>, Vec<Version>, Version) {
        (
            vec![
                Version::default(),
                Version {
                    identifier: "3".to_string(),
                    features: Vec::new(),
                },
                Version {
                    identifier: "4".to_string(),
                    features: Vec::new(),
                },
            ]
            .into_iter()
            .collect(),
            vec![
                Version {
                    identifier: "2".to_string(),
                    features: Vec::new(),
                },
                Version {
                    identifier: "4".to_string(),
                    features: Vec::new(),
                },
                Version {
                    identifier: "3".to_string(),
                    features: Vec::new(),
                },
            ]
            .into_iter()
            .collect(),
            // Should pick version 3 as it's the lowest of the intersection {3, 4}
            Version {
                identifier: "3".to_string(),
                features: Vec::new(),
            },
        )
    }

    fn disjoint() -> (Vec<Version>, Vec<Version>) {
        (
            vec![Version {
                identifier: "1".to_string(),
                features: Vec::new(),
            }]
            .into_iter()
            .collect(),
            vec![Version {
                identifier: "2".to_string(),
                features: Vec::new(),
            }]
            .into_iter()
            .collect(),
        )
    }

    #[test]
    fn verify() {
        struct Test {
            name: String,
            versions: Vec<RawVersion>,
            want_pass: bool,
        }
        let tests: Vec<Test> = vec![
            Test {
                name: "Compatible versions".to_string(),
                versions: vec![Version::default().into()],
                want_pass: true,
            },
            Test {
                name: "Multiple versions".to_string(),
                versions: good_versions(),
                want_pass: true,
            },
            Test {
                name: "Bad version identifier".to_string(),
                versions: bad_versions_identifier(),
                want_pass: false,
            },
            Test {
                name: "Bad version feature".to_string(),
                versions: bad_versions_features(),
                want_pass: false,
            },
            Test {
                name: "Bad versions empty".to_string(),
                versions: Vec::new(),
                want_pass: true,
            },
        ];

        for test in tests {
            let versions = test
                .versions
                .into_iter()
                .map(Version::try_from)
                .collect::<Result<Vec<_>, _>>();

            assert_eq!(
                test.want_pass,
                versions.is_ok(),
                "Validate versions failed for test {} with error {:?}",
                test.name,
                versions.err(),
            );
        }
    }
    #[test]
    fn pick() {
        struct Test {
            name: String,
            supported: Vec<Version>,
            counterparty: Vec<Version>,
            picked: Result<Version, ConnectionError>,
            want_pass: bool,
        }
        let tests: Vec<Test> = vec![
            Test {
                name: "Compatible versions".to_string(),
                supported: Version::compatible_versions(),
                counterparty: Version::compatible_versions(),
                picked: Ok(Version::default()),
                want_pass: true,
            },
            Test {
                name: "Overlapping versions".to_string(),
                supported: overlapping().0,
                counterparty: overlapping().1,
                picked: Ok(overlapping().2),
                want_pass: true,
            },
            Test {
                name: "Disjoint versions".to_string(),
                supported: disjoint().0,
                counterparty: disjoint().1,
                picked: Err(ConnectionError::NoCommonVersion),
                want_pass: false,
            },
        ];

        for test in tests {
            let version = Version::select(&test.supported, &test.counterparty);

            assert_eq!(
                test.want_pass,
                version.is_ok(),
                "Validate versions failed for test {}",
                test.name,
            );

            if test.want_pass {
                assert_eq!(version.unwrap(), test.picked.unwrap());
            }
        }
    }
    #[test]
    fn serialize() {
        let def = Version::default();
        let def_raw: RawVersion = def.clone().into();
        let def_back = def_raw.try_into().unwrap();
        assert_eq!(def, def_back);
    }
}
