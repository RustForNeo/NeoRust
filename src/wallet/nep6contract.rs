use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct NEP6Contract {
    pub script: Option<String>,

    #[serde(rename = "deployed")]
    pub is_deployed: bool,

    #[serde(rename = "parameters")]
    pub nep6_parameters: Vec<NEP6Parameter>,
}

#[derive(Serialize, Deserialize)]
pub struct NEP6Parameter {

    #[serde(rename = "name")]
    pub param_name: String,

    pub param_type: ContractParamType,

}

impl PartialEq for NEP6Contract {
    fn eq(&self, other: &Self) -> bool {
        self.script == other.script
            && self.nep6_parameters == other.nep6_parameters
            && self.is_deployed == other.is_deployed
    }
}