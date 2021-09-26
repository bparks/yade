use yaml_rust::Yaml::Integer;
use std::path::Path;
use std::fs;
use std::iter::Iterator;
use std::io::Read;
use yaml_rust::YamlLoader;

pub struct ResultSet {
    pub fields: Vec<String>,
    pub rows: Option<Vec<Vec<String>>>
}

pub fn list_files(path: &String) -> ResultSet {
    if Path::new(&path).exists() {
        let mut file = fs::File::open(format!("{}/{}", path, "schema.yml")).expect("Unable to open file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Unable to read file");
        let schema = &YamlLoader::load_from_str(&contents).unwrap()[0];
        let schema_hash = schema.as_hash().unwrap();
        let mut rs = ResultSet{
            fields: schema_hash.keys().map(|k| k.clone().into_string().unwrap()).collect(),
            rows: None
        };

        /*for f in &rs.fields {
            println!("{:?}", f);
        }*/

        rs.rows = Some(fs::read_dir(path).unwrap()
            .filter_map(|f| f.ok()
                .and_then(|e| e.path().file_name()
                    .and_then(|n| {
                        //println!("{:?}", &n);

                        if n != "schema.yml" {

                            let mut rfile = fs::File::open(format!("{}/{}", path, n.to_str().map(|s| String::from(s)).unwrap())).expect("Unable to open file");
                            let mut rcontents = String::new();
                            rfile.read_to_string(&mut rcontents).expect("Unable to read file");
                            let record = &YamlLoader::load_from_str(&rcontents).unwrap()[0];

                            return Some(rs.fields.iter().map(|f| stringify(&record[f.as_str()])).collect::<Vec<String>>());
                        } else { None }
                    })))
            .collect());
            
        return rs;
    } else {
        return ResultSet { fields: Vec::new(), rows: None };
    }
}

fn stringify(yval: &yaml_rust::Yaml) -> String {
    match yval {
        Integer(i) => i.to_string(),
        s => s.as_str().unwrap_or("").to_string()
    }
}