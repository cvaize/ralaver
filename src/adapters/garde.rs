use std::collections::HashMap;
use garde::Report;

pub struct GardeReportAdapter<'a> {
    report: &'a Report,
}

impl<'a> GardeReportAdapter<'a> {
    pub fn new(report: &'a Report) -> Self {
        Self { report }
    }

    pub fn to_hash_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for (path, error) in self.report.iter() {
            map.insert(path.to_string(), error.message().to_string());
        }
        map
    }
}
