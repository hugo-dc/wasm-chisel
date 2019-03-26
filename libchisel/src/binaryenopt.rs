use super::{ModuleError, ModulePreset, ModuleTranslator};
use parity_wasm::elements::*;

// FIXME: change level names
pub enum BinaryenOptimiser {
    O0, // Baseline aka no changes
    O1,
    O2,
    O3,
    O4,
    Os,
    Oz,
}

impl ModulePreset for BinaryenOptimiser {
    fn with_preset(preset: &str) -> Result<Self, ()> {
        match preset {
            "O0" => Ok(BinaryenOptimiser::O0),
            "O1" => Ok(BinaryenOptimiser::O1),
            "O2" => Ok(BinaryenOptimiser::O2),
            "O3" => Ok(BinaryenOptimiser::O3),
            "O4" => Ok(BinaryenOptimiser::O4),
            "Os" => Ok(BinaryenOptimiser::Os),
            "Oz" => Ok(BinaryenOptimiser::Oz),
            _ => Err(()),
        }
    }
}

impl ModuleTranslator for BinaryenOptimiser {
    fn translate_inplace(&self, module: &mut Module) -> Result<bool, ModuleError> {
        Err(ModuleError::NotSupported)
    }

    fn translate(&self, module: &Module) -> Result<Option<Module>, ModuleError> {
        // FIXME: it may not be parsed yet and would need to call `module.parse_names();` first
        let has_names_section = module.names_section().is_some();

        // FIXME: could just move this into `BinaryenOptimiser`
        let config = match &self {
            BinaryenOptimiser::O0 => binaryen::CodegenConfig {
                optimization_level: 0,
                shrink_level: 0,
                debug_info: has_names_section,
            },
            BinaryenOptimiser::O1 => binaryen::CodegenConfig {
                optimization_level: 1,
                shrink_level: 0,
                debug_info: has_names_section,
            },
            BinaryenOptimiser::O2 => binaryen::CodegenConfig {
                optimization_level: 2,
                shrink_level: 0,
                debug_info: has_names_section,
            },
            BinaryenOptimiser::O3 => binaryen::CodegenConfig {
                optimization_level: 3,
                shrink_level: 0,
                debug_info: has_names_section,
            },
            BinaryenOptimiser::O4 => binaryen::CodegenConfig {
                optimization_level: 4,
                shrink_level: 0,
                debug_info: has_names_section,
            },
            BinaryenOptimiser::Os => binaryen::CodegenConfig {
                optimization_level: 2,
                shrink_level: 1,
                debug_info: has_names_section,
            },
            BinaryenOptimiser::Oz => binaryen::CodegenConfig {
                optimization_level: 2,
                shrink_level: 2,
                debug_info: has_names_section,
            },
        };

        // FIXME: there must be a better way to accomplish this.
        let serialised = parity_wasm::elements::serialize::<Module>(module.clone())
            .expect("invalid input module");
        let output = binaryen_optimiser(&serialised, &config)?;
        Ok(Some(
            parity_wasm::elements::deserialize_buffer::<Module>(&output[..])
                .expect("invalid output module"),
        ))
    }
}

fn binaryen_optimiser(
    input: &[u8],
    config: &binaryen::CodegenConfig,
) -> Result<Vec<u8>, ModuleError> {
    match binaryen::Module::read(&input) {
        Ok(module) => {
            // NOTE: this is a global setting...
            binaryen::set_global_codegen_config(&config);
            module.optimize();
            Ok(module.write())
        }
        Err(_) => Err(ModuleError::Custom(
            "Failed to deserialise binary with binaryen".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_wasm::elements::deserialize_buffer;

    #[test]
    fn smoke_test_o0() {
        let input: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
            0x08, 0x01, 0x00, 0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b,
        ];

        let expected: Vec<u8> = vec![
            0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00, 0x01, 0x04, 0x01, 0x60, 0x00, 0x00,
            0x03, 0x02, 0x01, 0x00, 0x07, 0x08, 0x01, 0x04, 0x6d, 0x61, 0x69, 0x6e, 0x00, 0x00,
            0x0a, 0x05, 0x01, 0x03, 0x00, 0x01, 0x0b,
        ];

        let module = deserialize_buffer::<Module>(&input).unwrap();
        let translator = BinaryenOptimiser::with_preset("O0").unwrap();
        let result = translator.translate(&module).unwrap().unwrap();
        let serialised = parity_wasm::elements::serialize::<Module>(result.clone())
            .expect("invalid input module");
        assert_eq!(expected, serialised);
    }
}
