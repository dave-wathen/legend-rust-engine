// // Copyright 2022 Dave Wathen. All rights reserved.

use std::fmt;

use crate::*;

pub const ROOT: ElementId = ElementId(0);
pub const PACKAGE_SEPARATOR: &str = "::";

pub struct Model
{
    elements: Vec<ElementData>,
}

impl fmt::Debug for Model
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { f.debug_struct("Model").field("elements_count", &self.elements.len()).finish() }
}

impl Model
{
    pub fn new() -> Self { Model { elements: vec![ElementData { id: ROOT, name: "Root".to_string(), parent: ROOT, children: vec![] }] } }

    pub fn add(&mut self, full_name: &str) -> PureExecutionResult<ElementId> { self.add_to(full_name, ROOT) }

    fn add_to(&mut self, full_name: &str, parent_id: ElementId) -> PureExecutionResult<ElementId>
    {
        if let Some(idx) = full_name.find(PACKAGE_SEPARATOR)
        {
            let head = &full_name[..idx];
            let tail = &full_name[(idx + 2)..];
            let child_id = self.find_child_id(head, parent_id).unwrap_or_else(|| self.create(head, parent_id));
            self.add_to(tail, child_id)
        }
        else if let Some(existing_id) = self.find_child_id(full_name, parent_id)
        {
            Err(PureExecutionError::DuplicateElementName { name: self.path(existing_id) })
        }
        else
        {
            Ok(self.create(full_name, parent_id))
        }
    }

    fn create(&mut self, name: &str, parent_id: ElementId) -> ElementId
    {
        let new_id = ElementId(self.elements.len());
        self.elements.push(ElementData { id: new_id, name: name.to_owned(), parent: parent_id, children: vec![] });
        self.elements[parent_id.0].children.push(new_id);
        new_id
    }

    pub fn get_element(&self, id: &ElementId) -> Element
    {
        let data = &self.elements[id.0];
        Element { model: self, data }
    }

    pub fn get_element_by_name(&self, full_name: &str) -> Option<Element> { self.find(full_name, ROOT) }

    fn find(&self, full_name: &str, parent_id: ElementId) -> Option<Element>
    {
        if let Some(idx) = full_name.find(PACKAGE_SEPARATOR)
        {
            let head = &full_name[..idx];
            let tail = &full_name[(idx + 2)..];
            let child_id = self.find_child_id(head, parent_id)?;
            self.find(tail, child_id)
        }
        else
        {
            let id = self.find_child_id(full_name, parent_id)?;
            Some(self.get_element(&id))
        }
    }

    fn find_child_id(&self, name: &str, parent_id: ElementId) -> Option<ElementId>
    {
        self.elements[parent_id.0].children.iter().find(|child_id| self.elements[child_id.0].name == name).copied()
    }

    fn path(&self, id: ElementId) -> String
    {
        let data = &self.elements[id.0];

        if id == ROOT
        {
            "".to_owned()
        }
        else if data.parent == ROOT
        {
            data.name.clone()
        }
        else
        {
            self.path(data.parent) + PACKAGE_SEPARATOR + &data.name
        }
    }
}

impl Default for Model
{
    fn default() -> Self { Self::new() }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ElementId(usize);

#[derive(Debug)]
pub struct ElementData
{
    id: ElementId,
    name: String,
    parent: ElementId,
    children: Vec<ElementId>,
}

#[derive(Copy, Clone, Debug)]
pub struct Element<'model>
{
    model: &'model Model,
    data: &'model ElementData,
}

impl Element<'_>
{
    pub fn id(&self) -> ElementId { self.data.id }

    pub fn name(&self) -> &str { &self.data.name }

    pub fn path(&self) -> String { self.model.path(self.data.id) }

    pub fn parent(&self) -> Element
    {
        let data = &self.model.elements[self.data.parent.0];
        Element { model: self.model, data }
    }

    pub fn children(&self) -> Vec<Element> { self.data.children.iter().map(|id| self.model.get_element(id)).collect() }
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    fn basic_model() -> PureExecutionResult<()>
    {
        let mut model = Model::new();
        let firm_id = model.add("domain::hr::Firm")?;
        model.add("domain::hr::Employee")?;
        model.add("domain::hr::Role")?;
        model.add("domain::ref::Address")?;

        let firm = model.get_element(&firm_id);
        assert_eq!(firm_id, firm.id());
        assert_eq!("Firm", firm.name());
        assert_eq!("domain::hr::Firm", firm.path());

        let hr = firm.parent();
        assert_eq!("hr", hr.name());
        assert_eq!("domain::hr", hr.path());
        assert_eq!(3, hr.children().len());

        assert!(model.get_element_by_name("missing").is_none());

        let domain_ref = model.get_element_by_name("domain::ref");
        assert!(domain_ref.is_some());
        assert_eq!(1, domain_ref.unwrap().children().len());

        Ok(())
    }

    #[test]
    fn cannot_add_duplicate() -> PureExecutionResult<()>
    {
        let mut model = Model::new();
        model.add("domain::hr::Firm")?;
        let err = model.add("domain::hr::Firm");
        assert!(err.is_err());
        assert_eq!("DuplicateElementName: domain::hr::Firm", format!("{}", err.err().unwrap()));
        Ok(())
    }
}
