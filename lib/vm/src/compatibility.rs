use parity_wasm::elements::{External, ImportEntry, Module};
use std::collections::BTreeSet;
use std::collections::HashSet;

use crate::capabilities::required_capabilities_from_module;
use crate::errors::{VmError, VmResult};
use crate::limited::LimitedDisplay;
use crate::static_analysis::{deserialize_wasm, ExportInfo};

/// Lists all imports we provide upon instantiating the instance in Instance::from_module()
/// This should be updated when new imports are added
const SUPPORTED_IMPORTS: &[&str] = &["env.abort", "env.debug", "env.read"];

/// Lists all entry points we expect to be present when calling a contract.
/// Other optional exports exist, e.g. "execute", "migrate" and "query".
/// The marker export interface_version_* is checked separately.
/// This is unlikely to change much, must be frozen at 1.0 to avoid breaking existing contracts
const REQUIRED_EXPORTS: &[&str] = &[
    // IO
    "allocate",
    "deallocate",
    // Required entry points
    "fetch",
];

const INTERFACE_VERSION_PREFIX: &str = "interface_version_";
const SUPPORTED_INTERFACE_VERSIONS: &[&str] = &["interface_version_8"];

const MEMORY_LIMIT: u32 = 512; // in pages

/// Checks if the data is valid wasm and compatibility with the CosmWasm API (imports and exports)
pub fn check_wasm(wasm_code: &[u8], available_capabilities: &HashSet<String>) -> VmResult<()> {
    let module = deserialize_wasm(wasm_code)?;
    check_wasm_memories(&module)?;
    check_interface_version(&module)?;
    check_wasm_exports(&module)?;
    check_wasm_imports(&module, SUPPORTED_IMPORTS)?;
    check_wasm_capabilities(&module, available_capabilities)?;
    Ok(())
}

fn check_wasm_memories(module: &Module) -> VmResult<()> {
    let section = match module.memory_section() {
        Some(section) => section,
        None => {
            return Err(VmError::static_validation_err(
                "Wasm contract doesn't have a memory section",
            ));
        }
    };

    let memories = section.entries();
    if memories.len() != 1 {
        return Err(VmError::static_validation_err(
            "Wasm contract must contain exactly one memory",
        ));
    }

    let memory = memories[0];
    // println!("Memory: {:?}", memory);
    let limits = memory.limits();

    if limits.initial() > MEMORY_LIMIT {
        return Err(VmError::static_validation_err(format!(
            "Wasm contract memory's minimum must not exceed {} pages.",
            MEMORY_LIMIT
        )));
    }

    if limits.maximum().is_some() {
        return Err(VmError::static_validation_err(
            "Wasm contract memory's maximum must be unset. The host will set it for you.",
        ));
    }
    Ok(())
}

fn check_interface_version(module: &Module) -> VmResult<()> {
    let mut interface_version_exports = module
        .exported_function_names(Some(INTERFACE_VERSION_PREFIX))
        .into_iter();
    if let Some(first_interface_version_export) = interface_version_exports.next() {
        if interface_version_exports.next().is_some() {
            Err(VmError::static_validation_err(
                "Wasm contract contains more than one marker export: interface_version_*",
            ))
        } else {
            // Exactly one interface version found
            let version_str = first_interface_version_export.as_str();
            if SUPPORTED_INTERFACE_VERSIONS
                .iter()
                .any(|&v| v == version_str)
            {
                Ok(())
            } else {
                Err(VmError::static_validation_err(
                    "Wasm contract has unknown interface_version_* marker export",
                ))
            }
        }
    } else {
        Err(VmError::static_validation_err(
            "Wasm contract missing a required marker export: interface_version_*",
        ))
    }
}

fn check_wasm_exports(module: &Module) -> VmResult<()> {
    let available_exports: HashSet<String> = module.exported_function_names(None);
    for required_export in REQUIRED_EXPORTS {
        if !available_exports.contains(*required_export) {
            return Err(VmError::static_validation_err(format!(
                "Wasm contract doesn't have required export: \"{}\". Exports required by VM: {:?}.",
                required_export, REQUIRED_EXPORTS
            )));
        }
    }
    Ok(())
}

/// Checks if the import requirements of the contract are satisfied.
/// When this is not the case, we either have an incompatibility between contract and VM
/// or a error in the contract.
fn check_wasm_imports(module: &Module, supported_imports: &[&str]) -> VmResult<()> {
    let required_imports: Vec<ImportEntry> = module
        .import_section()
        .map_or(vec![], |import_section| import_section.entries().to_vec());
    let required_import_names: BTreeSet<_> =
        required_imports.iter().map(full_import_name).collect();

    for required_import in required_imports {
        let full_name = full_import_name(&required_import);
        if !supported_imports.contains(&full_name.as_str()) {
            return Err(VmError::static_validation_err(format!(
                "Wasm contract requires unsupported import: \"{}\". Required imports: {}. Available imports: {:?}.",
                full_name, required_import_names.to_string_limited(200), supported_imports
            )));
        }

        match required_import.external() {
            External::Function(_) => {}, // ok
            _ => return Err(VmError::static_validation_err(format!(
                "Wasm contract requires non-function import: \"{}\". Right now, all supported imports are functions.",
                full_name
            ))),
        };
    }
    Ok(())
}

fn full_import_name(ie: &ImportEntry) -> String {
    format!("{}.{}", ie.module(), ie.field())
}

fn check_wasm_capabilities(
    module: &Module,
    available_capabilities: &HashSet<String>,
) -> VmResult<()> {
    let required_capabilities = required_capabilities_from_module(module);
    if !required_capabilities.is_subset(available_capabilities) {
        // We switch to BTreeSet to get a sorted error message
        let unavailable: BTreeSet<_> = required_capabilities
            .difference(available_capabilities)
            .collect();
        return Err(VmError::static_validation_err(format!(
            "Wasm contract requires unavailable capabilities: {}",
            unavailable.to_string_limited(200)
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::VmError;

    static CONTRACT: &[u8] = include_bytes!("../testdata/json.wasm");

    fn default_capabilities() -> HashSet<String> {
        ["staking".to_string()].into_iter().collect()
    }

    #[test]
    fn check_wasm_passes_for_latest_contract() {
        // this is our reference check, must pass
        check_wasm(CONTRACT, &default_capabilities()).unwrap();
    }

    #[test]
    fn check_wasm_memories_ok() {
        let wasm = wat::parse_str("(module (memory 1))").unwrap();
        check_wasm_memories(&deserialize_wasm(&wasm).unwrap()).unwrap()
    }

    #[test]
    fn check_wasm_memories_no_memory() {
        let wasm = wat::parse_str("(module)").unwrap();
        match check_wasm_memories(&deserialize_wasm(&wasm).unwrap()) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.starts_with("Wasm contract doesn't have a memory section"));
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn check_wasm_memories_two_memories() {
        // Generated manually because wat2wasm protects us from creating such Wasm:
        // "error: only one memory block allowed"
        let wasm = hex::decode(concat!(
            "0061736d", // magic bytes
            "01000000", // binary version (uint32)
            "05",       // section type (memory)
            "05",       // section length
            "02",       // number of memories
            "0009",     // element of type "resizable_limits", min=9, max=unset
            "0009",     // element of type "resizable_limits", min=9, max=unset
        ))
        .unwrap();

        match check_wasm_memories(&deserialize_wasm(&wasm).unwrap()) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.starts_with("Wasm contract must contain exactly one memory"));
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn check_wasm_memories_zero_memories() {
        // Generated manually because wat2wasm would not create an empty memory section
        let wasm = hex::decode(concat!(
            "0061736d", // magic bytes
            "01000000", // binary version (uint32)
            "05",       // section type (memory)
            "01",       // section length
            "00",       // number of memories
        ))
        .unwrap();

        match check_wasm_memories(&deserialize_wasm(&wasm).unwrap()) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.starts_with("Wasm contract must contain exactly one memory"));
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn check_wasm_memories_initial_size() {
        let wasm_ok = wat::parse_str("(module (memory 512))").unwrap();
        check_wasm_memories(&deserialize_wasm(&wasm_ok).unwrap()).unwrap();

        let wasm_too_big = wat::parse_str("(module (memory 513))").unwrap();
        match check_wasm_memories(&deserialize_wasm(&wasm_too_big).unwrap()) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.starts_with("Wasm contract memory's minimum must not exceed 512 pages"));
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn check_wasm_memories_maximum_size() {
        let wasm_max = wat::parse_str("(module (memory 1 5))").unwrap();
        match check_wasm_memories(&deserialize_wasm(&wasm_max).unwrap()) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.starts_with("Wasm contract memory's maximum must be unset"));
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn check_interface_version_works() {
        // valid
        let wasm = wat::parse_str(
            r#"(module
                (type (func))
                (func (type 0) nop)
                (export "add_one" (func 0))
                (export "allocate" (func 0))
                (export "interface_version_8" (func 0))
                (export "deallocate" (func 0))
                (export "fetch" (func 0))
            )"#,
        )
        .unwrap();
        let module = deserialize_wasm(&wasm).unwrap();
        check_interface_version(&module).unwrap();

        #[cfg(feature = "allow_interface_version_7")]
        {
            // valid legacy version
            let wasm = wat::parse_str(
                r#"(module
                    (type (func))
                    (func (type 0) nop)
                    (export "add_one" (func 0))
                    (export "allocate" (func 0))
                    (export "interface_version_7" (func 0))
                    (export "deallocate" (func 0))
                    (export "fetch" (func 0))
                )"#,
            )
            .unwrap();
            let module = deserialize_wasm(&wasm).unwrap();
            check_interface_version(&module).unwrap();
        }

        // missing
        let wasm = wat::parse_str(
            r#"(module
                (type (func))
                (func (type 0) nop)
                (export "add_one" (func 0))
                (export "allocate" (func 0))
                (export "deallocate" (func 0))
                (export "fetch" (func 0))
            )"#,
        )
        .unwrap();
        let module = deserialize_wasm(&wasm).unwrap();
        match check_interface_version(&module).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => {
                assert_eq!(
                    msg,
                    "Wasm contract missing a required marker export: interface_version_*"
                );
            }
            err => panic!("Unexpected error {:?}", err),
        }

        // multiple
        let wasm = wat::parse_str(
            r#"(module
                (type (func))
                (func (type 0) nop)
                (export "add_one" (func 0))
                (export "allocate" (func 0))
                (export "interface_version_8" (func 0))
                (export "interface_version_9" (func 0))
                (export "deallocate" (func 0))
                (export "fetch" (func 0))
            )"#,
        )
        .unwrap();
        let module = deserialize_wasm(&wasm).unwrap();
        match check_interface_version(&module).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => {
                assert_eq!(
                    msg,
                    "Wasm contract contains more than one marker export: interface_version_*"
                );
            }
            err => panic!("Unexpected error {:?}", err),
        }

        // CosmWasm 0.15
        let wasm = wat::parse_str(
            r#"(module
                (type (func))
                (func (type 0) nop)
                (export "add_one" (func 0))
                (export "allocate" (func 0))
                (export "interface_version_6" (func 0))
                (export "deallocate" (func 0))
                (export "fetch" (func 0))
            )"#,
        )
        .unwrap();
        let module = deserialize_wasm(&wasm).unwrap();
        match check_interface_version(&module).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => {
                assert_eq!(
                    msg,
                    "Wasm contract has unknown interface_version_* marker export"
                );
            }
            err => panic!("Unexpected error {:?}", err),
        }

        // Unknown value
        let wasm = wat::parse_str(
            r#"(module
                (type (func))
                (func (type 0) nop)
                (export "add_one" (func 0))
                (export "allocate" (func 0))
                (export "interface_version_broken" (func 0))
                (export "deallocate" (func 0))
                (export "fetch" (func 0))
            )"#,
        )
        .unwrap();
        let module = deserialize_wasm(&wasm).unwrap();
        match check_interface_version(&module).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => {
                assert_eq!(
                    msg,
                    "Wasm contract has unknown interface_version_* marker export"
                );
            }
            err => panic!("Unexpected error {:?}", err),
        }
    }

    #[test]
    fn check_wasm_exports_works() {
        // valid
        let wasm = wat::parse_str(
            r#"(module
                (type (func))
                (func (type 0) nop)
                (export "add_one" (func 0))
                (export "allocate" (func 0))
                (export "deallocate" (func 0))
                (export "fetch" (func 0))
            )"#,
        )
        .unwrap();
        let module = deserialize_wasm(&wasm).unwrap();
        check_wasm_exports(&module).unwrap();

        // this is invalid, as it doesn't any required export
        let wasm = wat::parse_str(
            r#"(module
                (type (func))
                (func (type 0) nop)
                (export "add_one" (func 0))
            )"#,
        )
        .unwrap();
        let module = deserialize_wasm(&wasm).unwrap();
        match check_wasm_exports(&module) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(msg.starts_with("Wasm contract doesn't have required export: \"allocate\""));
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }

        // this is invalid, as it doesn't contain all required exports
        let wasm = wat::parse_str(
            r#"(module
                (type (func))
                (func (type 0) nop)
                (export "add_one" (func 0))
                (export "allocate" (func 0))
            )"#,
        )
        .unwrap();
        let module = deserialize_wasm(&wasm).unwrap();
        match check_wasm_exports(&module) {
            Err(VmError::StaticValidationErr { msg, .. }) => {
                assert!(
                    msg.starts_with("Wasm contract doesn't have required export: \"deallocate\"")
                );
            }
            Err(e) => panic!("Unexpected error {:?}", e),
            Ok(_) => panic!("Didn't reject wasm with invalid api"),
        }
    }

    #[test]
    fn check_wasm_imports_ok() {
        let wasm = wat::parse_str(
            r#"(module
            (import "env" "read" (func (param i32) (result i32)))
        )"#,
        )
        .unwrap();
        check_wasm_imports(&deserialize_wasm(&wasm).unwrap(), SUPPORTED_IMPORTS).unwrap();
    }

    #[test]
    fn check_wasm_imports_missing() {
        let wasm = wat::parse_str(
            r#"(module
            (import "env" "foo" (func (param i32 i32) (result i32)))
            (import "env" "bar" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam01" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam02" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam03" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam04" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam05" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam06" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam07" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam08" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam09" (func (param i32 i32) (result i32)))
            (import "env" "spammyspam10" (func (param i32 i32) (result i32)))
        )"#,
        )
        .unwrap();
        let supported_imports: &[&str] = &["env.read", "env.debug"];
        let result = check_wasm_imports(&deserialize_wasm(&wasm).unwrap(), supported_imports);
        match result.unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => {
                println!("{}", msg);
                assert_eq!(
                    msg,
                    r#"Wasm contract requires unsupported import: "env.foo". Required imports: {"env.bar", "env.foo", "env.spammyspam01", "env.spammyspam02", "env.spammyspam03", "env.spammyspam04", "env.spammyspam05", "env.spammyspam06", "env.spammyspam07", "env.spammyspam08", ... 2 more}. Available imports: ["env.read", "env.debug"]."#
                );
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn check_wasm_imports_wrong_type() {
        let wasm = wat::parse_str(r#"(module (import "env" "read" (memory 1 1)))"#).unwrap();
        let result = check_wasm_imports(&deserialize_wasm(&wasm).unwrap(), SUPPORTED_IMPORTS);
        match result.unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => {
                assert!(msg.starts_with("Wasm contract requires non-function import: \"env.read\""));
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn check_wasm_capabilities_ok() {
        let wasm = wat::parse_str(
            r#"(module
            (type (func))
            (func (type 0) nop)
            (export "requires_water" (func 0))
            (export "requires_" (func 0))
            (export "requires_nutrients" (func 0))
            (export "require_milk" (func 0))
            (export "REQUIRES_air" (func 0))
            (export "requires_sun" (func 0))
        )"#,
        )
        .unwrap();
        let module = deserialize_wasm(&wasm).unwrap();
        let available = [
            "water".to_string(),
            "nutrients".to_string(),
            "sun".to_string(),
            "freedom".to_string(),
        ]
        .into_iter()
        .collect();
        check_wasm_capabilities(&module, &available).unwrap();
    }

    #[test]
    fn check_wasm_capabilities_fails_for_missing() {
        let wasm = wat::parse_str(
            r#"(module
            (type (func))
            (func (type 0) nop)
            (export "requires_water" (func 0))
            (export "requires_" (func 0))
            (export "requires_nutrients" (func 0))
            (export "require_milk" (func 0))
            (export "REQUIRES_air" (func 0))
            (export "requires_sun" (func 0))
        )"#,
        )
        .unwrap();
        let module = deserialize_wasm(&wasm).unwrap();

        // Available set 1
        let available = [
            "water".to_string(),
            "nutrients".to_string(),
            "freedom".to_string(),
        ]
        .into_iter()
        .collect();
        match check_wasm_capabilities(&module, &available).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => assert_eq!(
                msg,
                "Wasm contract requires unavailable capabilities: {\"sun\"}"
            ),
            _ => panic!("Got unexpected error"),
        }

        // Available set 2
        let available = [
            "nutrients".to_string(),
            "freedom".to_string(),
            "Water".to_string(), // capabilities are case sensitive (and lowercase by convention)
        ]
        .into_iter()
        .collect();
        match check_wasm_capabilities(&module, &available).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => assert_eq!(
                msg,
                "Wasm contract requires unavailable capabilities: {\"sun\", \"water\"}"
            ),
            _ => panic!("Got unexpected error"),
        }

        // Available set 3
        let available = ["freedom".to_string()].into_iter().collect();
        match check_wasm_capabilities(&module, &available).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => assert_eq!(
                msg,
                "Wasm contract requires unavailable capabilities: {\"nutrients\", \"sun\", \"water\"}"
            ),
            _ => panic!("Got unexpected error"),
        }

        // Available set 4
        let available = [].into_iter().collect();
        match check_wasm_capabilities(&module, &available).unwrap_err() {
            VmError::StaticValidationErr { msg, .. } => assert_eq!(
                msg,
                "Wasm contract requires unavailable capabilities: {\"nutrients\", \"sun\", \"water\"}"
            ),
            _ => panic!("Got unexpected error"),
        }
    }
}
