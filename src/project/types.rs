use serde::Serialize;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Field {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Component {
    pub kind: String,
    pub name: String,
    pub filename: Option<String>,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProjectInfo {
    pub name: String,
    pub description: String,
    pub version: String,
    pub dss_file: String,
    pub time_zone: String,
    pub components: Vec<Component>,
    pub controls: Vec<ControlSpec>,
    pub meteorology: Vec<MetInfo>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ControlSpec {
    pub name: String,
    pub start_date: String,
    pub start_time: String,
    pub end_date: String,
    pub end_time: String,
    pub interval: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MetInfo {
    pub name: String,
    pub methods: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Element {
    pub kind: String,
    pub name: String,
    pub downstream: Option<String>,
    pub area: Option<String>,
    pub parameters: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BasinInfo {
    pub name: String,
    pub unit_system: String,
    pub version: String,
    pub elements: Vec<Element>,
}
