use crate::sql::Value;
use yaml_rust::Yaml;
use crate::sql::Predicate;
use crate::sql::Setter;
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

fn get_schema(path: &String) -> yaml_rust::yaml::Hash {
    let mut file = fs::File::open(format!("{}/{}", path, "schema.yml")).expect("Unable to open file");
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("Unable to read file");
    let schema = &YamlLoader::load_from_str(&contents).unwrap()[0];
    return schema.as_hash().unwrap().clone();
}

pub fn list_files(path: &String) -> ResultSet {
    if Path::new(&path).exists() {
        let schema_hash = get_schema(path);
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
                            let filename = format!("{}/{}", path, n.to_str().map(|s| String::from(s)).unwrap());
                            let record = open_yaml_file(&filename);
                            return Some(rs.fields.iter().map(|f| stringify(&record[f.as_str()])).collect::<Vec<String>>());
                        } else { None }
                    })))
            .collect());
            
        return rs;
    } else {
        return ResultSet { fields: Vec::new(), rows: None };
    }
}

/*fn filterBy<'a>(predicate: &Option<Predicate>) -> Box<dyn Fn(&String) + bool> {
    return match predicate {
        Some(real_pred) => Box::new(move |s| {
            true
        }),
        None => Box::new(|s| false)
    }
}*/

fn open_yaml_file(s: &String) -> Yaml {
    let mut rfile = fs::File::open(s).expect(format!("Unable to open file {}", s).as_str());
    let mut rcontents = String::new();
    rfile.read_to_string(&mut rcontents).expect("Unable to read file");
    return YamlLoader::load_from_str(&rcontents).unwrap()[0].clone();
}

fn evaluate_value(y: &Yaml, value: &Value) -> String {
    return match value {
        Value::Column(column_name) => stringify(&y[column_name.as_str()]),
        Value::StringLiteral(val) => val.clone()
    }
}

fn evaluate_predicate(y: &Yaml, predicate: &Predicate) -> bool {
    return match predicate {
        Predicate::And(pred1, pred2) => evaluate_predicate(&y, pred1) && evaluate_predicate(&y, pred2),
        Predicate::Or(pred1, pred2) => evaluate_predicate(&y, pred1) || evaluate_predicate(&y, pred2),
        Predicate::Equals(value1, value2) => evaluate_value(&y, value1) == evaluate_value(&y, value2)
    }
}

pub fn update_files(path: &String, setters: &Option<Vec<Setter>>, predicate: &Option<Predicate>) -> u8 {
    if matches!(setters, None) { return 0; }
    let _schema_hash = get_schema(path);
    let files: Vec<String> = fs::read_dir(path).unwrap()
        .filter_map(|f| f.ok()
            .and_then(|e| e.path().file_name()
                .and_then(|n| {
                    if n == "schema.yml" { return None }
                    return Some(n.to_str().map(String::from).unwrap());
                })))
        .collect();
    return files.iter().filter(|s| {
        // TODO: I really want this to be a `filterBy(predicate)` function (see attempt above)
        let filename = format!("{}/{}", path, s);
        return match predicate {
            Some(real_predicate) => evaluate_predicate(&open_yaml_file(&filename), real_predicate),
            None => false
        }
    }).map(|_s| {
        // TODO: I really want this to be a `update(filters)` function
        //let filename = format!("{}/{}", path, s);
        return match setters {
            Some(_real_setters) => 1,
            None => 0 // Why would we even have no setters?
        };
    }).fold(0, |a, x| a + x);
}

fn stringify(yval: &yaml_rust::Yaml) -> String {
    match yval {
        Integer(i) => i.to_string(),
        s => s.as_str().unwrap_or("").to_string()
    }
}