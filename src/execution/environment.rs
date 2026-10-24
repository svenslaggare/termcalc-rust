use fnv::FnvHashMap;

use crate::model::{Function, ResultValue};
use crate::types::NumberTypeId;

#[derive(Clone)]
pub struct Environment {
    scopes: FnvHashMap<NumberTypeId, EnvironmentScope>,
    active: NumberTypeId
}

impl Environment {
    pub fn new(
        scopes: impl Iterator<Item=(NumberTypeId, EnvironmentScope)>,
        active: NumberTypeId
    ) -> Environment {
        Environment {
            scopes: FnvHashMap::from_iter(scopes),
            active
        }
    }

    pub fn empty(number_types: impl Iterator<Item=NumberTypeId>, active: NumberTypeId) -> Environment {
        Environment {
            scopes: FnvHashMap::from_iter(number_types.map(|number_type| (number_type, EnvironmentScope::new(Vec::new(), Vec::new())))),
            active
        }
    }

    pub fn active(&self) -> NumberTypeId {
        self.active
    }

    pub fn set_active(&mut self, value: NumberTypeId) {
        self.active = value;
    }

    pub fn scopes(&self) -> impl Iterator<Item=NumberTypeId> {
        self.scopes.keys().cloned()
    }

    pub fn scope(&self) -> &EnvironmentScope {
        &self.scopes[&self.active]
    }

    pub fn scope_mut(&mut self) -> &mut EnvironmentScope {
        self.scopes.get_mut(&self.active).unwrap()
    }

    pub fn get_variable(&self, name: &str) -> Option<ResultValue> {
        self.scope().get_variable(name)
    }

    pub fn variables(&self) -> impl Iterator<Item=(&String, &ResultValue)> {
        self.scope().variables()
    }

    pub fn define_variable(&mut self, name: &str, value: ResultValue) {
        for scope in self.scopes.values_mut() {
            scope.define_variable(name, value.clone());
        }
    }

    pub fn define_variables(&mut self, iterator: impl Iterator<Item=(String, ResultValue)>) {
        let variables = iterator.collect::<Vec<_>>();
        for scope in self.scopes.values_mut() {
            scope.define_variables(variables.clone().into_iter());
        }
    }

    pub fn get_function(&self, name: &str) -> Option<&Function> {
        self.scope().get_function(name)
    }

    pub fn functions(&self) -> impl Iterator<Item=(&String, &Function)> {
        self.scope().functions()
    }

    pub fn define_function(&mut self, name: &str, function: Function) {
        for scope in self.scopes.values_mut() {
            scope.define_function(name, function.clone());
        }
    }

    pub fn define_functions(&mut self, iterator: impl Iterator<Item=(String, Function)>) {
        let functions = iterator.collect::<Vec<_>>();
        for scope in self.scopes.values_mut() {
            scope.define_functions(functions.clone().into_iter());
        }
    }
}

#[derive(Clone)]
pub struct EnvironmentScope {
    variables: FnvHashMap<String, ResultValue>,
    functions: FnvHashMap<String, Function>
}

impl EnvironmentScope {
    pub fn new(variables: Vec<(String, ResultValue)>, functions: Vec<(String, Function)>) -> EnvironmentScope {
        EnvironmentScope {
            variables: FnvHashMap::from_iter(variables),
            functions: FnvHashMap::from_iter(functions)
        }
    }

    pub fn get_variable(&self, name: &str) -> Option<ResultValue> {
        self.variables.get(name).cloned()
    }

    pub fn variables(&self) -> impl Iterator<Item=(&String, &ResultValue)> {
        self.variables.iter()
    }

    pub fn define_variable(&mut self, name: &str, value: ResultValue) {
        self.variables.insert(name.to_owned(), value);
    }

    pub fn define_variables(&mut self, iterator: impl Iterator<Item=(String, ResultValue)>) {
        self.variables.extend(iterator);
    }

    pub fn get_function(&self, name: &str) -> Option<&Function> {
        self.functions.get(name)
    }

    pub fn functions(&self) -> impl Iterator<Item=(&String, &Function)> {
        self.functions.iter()
    }

    pub fn define_function(&mut self, name: &str, function: Function) {
        self.functions.insert(name.to_owned(), function);
    }

    pub fn define_functions(&mut self, iterator: impl Iterator<Item=(String, Function)>) {
        self.functions.extend(iterator);
    }
}