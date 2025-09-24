#[derive(Clone)]
pub struct TeamName {
    pub name: String,
    pub name_pretty: String,
}

impl TeamName {
    pub fn new(name: &str) -> Self {
        let name = name.to_string();
        let name_pretty = name.replace("_", " ");

        TeamName { name, name_pretty }
    }

    pub fn from_pretty(name_pretty: &str) -> Self {
        let name_pretty = name_pretty.to_string();
        let name = name_pretty.replace(" ", "_");

        TeamName { name, name_pretty }
    }
}
