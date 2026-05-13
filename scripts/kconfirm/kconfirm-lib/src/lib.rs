// SPDX-License-Identifier: GPL-2.0-only
use analyze::analyze;
pub use checks::AnalysisArgs;
pub use checks::Check;
pub use checks::check_select_visible;
pub use checks::check_variable_info;
use nom_kconfig::Entry;
use nom_kconfig::KconfigInput;
use nom_kconfig::parse_kconfig;
use output::*;
use symbol_table::*;
mod analyze;
mod checks;
mod curl_ffi;
mod dead_links;
pub mod output;
pub mod symbol_table;

pub fn check_kconfig(
    args: AnalysisArgs,
    kconfig_files: Vec<(String, KconfigInput)>,
) -> Vec<Finding> {
    let mut findings = Vec::new();
    let mut symbol_table = SymbolTable::new();

    for (arch_config_option, kconfig_file) in kconfig_files {
        match parse_kconfig(kconfig_file) {
            Ok(parsed) => {
                let entries: Vec<Entry> = parsed.1.entries;
                findings.extend(analyze(
                    &args,
                    &mut symbol_table,
                    arch_config_option,
                    entries,
                ));
            }
            Err(e) => {
                findings.push(Finding {
                    severity: Severity::Fatal,
                    check: Check::FailedParse,
                    symbol: None,
                    message: format!("Failed to parse kconfig, error is: {}", e),
                    arch: arch_config_option,
                });
            }
        }
    }

    for (var_symbol, type_info) in &symbol_table.raw {
        for (arch_specific, redefinitions) in &type_info.attribute_defs {
            for (_definition_condition, info) in redefinitions {
                findings.extend(check_variable_info(&args, var_symbol, arch_specific, info));
            }
        }

        if args.is_enabled(Check::SelectVisible) {
            findings.extend(check_select_visible(var_symbol, type_info));
        }
    }

    findings
}
