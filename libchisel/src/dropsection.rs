use super::{ModuleError, ModuleTranslator};

use parity_wasm::builder::*;
use parity_wasm::elements::*;

/// Enum on which ModuleTranslator is implemented.
pub enum DropSection<'a> {
    NamesSection,
    /// Name of the custom section.
    CustomSectionByName(&'a String),
    /// Index of the custom section.
    CustomSectionByIndex(usize),
    /// Index of the unknown section.
    UnknownSectionByIndex(usize),
}

fn names_section_index_for(module: &Module) -> Result<usize, String> {
    for (index, section) in module.sections().iter().enumerate() {
        if let Section::Name(_section) = section {
            return Ok(index);
        }
    }
    Err("Not found".to_string())
}

fn custom_section_index_for(module: &Module, name: &String) -> Result<usize, String> {
    for (index, section) in module.sections().iter().enumerate() {
        if let Section::Custom(_section) = section {
            if _section.name() == name {
                return Ok(index);
            }
        }
    }
    Err("Not found".to_string())
}

impl<'a> DropSection<'a> {
    fn find_index(&self, module: &Module) -> Result<usize, String> {
        Ok(match &self {
            DropSection::NamesSection => names_section_index_for(module)?,
            DropSection::CustomSectionByName(name) => custom_section_index_for(module, &name)?,
            DropSection::CustomSectionByIndex(index) => *index,
            DropSection::UnknownSectionByIndex(index) => *index,
        })
    }

    fn drop_section(&self, module: &mut Module) -> Result<bool, ModuleError> {
        let index = self.find_index(&module)?;

        let sections = module.sections_mut();
        if index > sections.len() {
            return Err(ModuleError::Custom("Not found.".to_string()));
        }
        sections.remove(index);

        Ok(true)
    }
}

impl<'a> ModuleTranslator for DropSection<'a> {
    fn translate_inplace(&self, module: &mut Module) -> Result<bool, ModuleError> {
        Ok(self.drop_section(module)?)
    }

    fn translate(&self, module: &Module) -> Result<Option<Module>, ModuleError> {
        let mut ret = module.clone();
        if self.drop_section(&mut ret)? {
            Ok(Some(ret))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_wasm::builder;

    #[test]
    fn keep_intact() {
        let mut module = builder::module().build();
        let name = "empty".to_string();
        let dropper = DropSection::CustomSectionByName(&name);
        let did_change = dropper.translate_inplace(&mut module).unwrap();
        assert_eq!(did_change, false);
    }

    #[test]
    fn keep_intact_custom_section() {
        let mut module = builder::module()
            .with_section(Section::Custom(CustomSection::new(
                "test".to_string(),
                vec![],
            )))
            .build();
        let name = "empty".to_string();
        let dropper = DropSection::CustomSectionByName(&name);
        let did_change = dropper.translate_inplace(&mut module).unwrap();
        assert_eq!(did_change, false);
    }

    #[test]
    fn remove_custom_section() {
        let mut module = builder::module()
            .with_section(Section::Custom(CustomSection::new(
                "test".to_string(),
                vec![],
            )))
            .build();
        let name = "test".to_string();
        let dropper = DropSection::CustomSectionByName(&name);
        let did_change = dropper.translate_inplace(&mut module).unwrap();
        assert_eq!(did_change, true);
    }
}
