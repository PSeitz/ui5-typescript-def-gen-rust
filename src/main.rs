#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate structopt;

use structopt::StructOpt;
use std::fs;
use std::fs::File;
use std::io::prelude::*;

#[derive(StructOpt, Debug)]
#[structopt(name = "upload")]
struct Opt {
    #[structopt(short = "r", long = "root_url", default_value = "https://sapui5.hana.ondemand.com/test-resources/")]
    root_url: String,
    /// comma-separated list of endpoints
    #[structopt(short = "endpoints", long = "endpoints", default_value = "sap/m/designtime/api.json,
      sap/ui/core/designtime/api.json,
      sap/tnt/designtime/api.json,
      sap/ui/commons/designtime/api.json,
      sap/ui/demokit/designtime/api.json,
      sap/ui/dt/designtime/api.json,
      sap/ui/layout/designtime/api.json,
      sap/ui/suite/designtime/api.json,
      sap/ui/table/designtime/api.json,
      sap/ui/unified/designtime/api.json,
      sap/ui/ux3/designtime/api.json,
      sap/uxap/designtime/api.json
    ", raw(use_delimiter = "true"))]
    endpoints: Vec<String>,
}

#[derive(Debug, Clone)]
struct FilePath {
    path: String,
    file_name: String
}

impl FilePath {
    fn full_path(&self) -> String {
        self.path.to_string() + "/" + &self.file_name
    }
}

fn main() -> Result<(), Box<std::error::Error>>{
    let opt = Opt::from_args();

    println!("{:?}", opt.endpoints.len());

    for endpoint in opt.endpoints {
        convert(&(opt.root_url.to_string() + &endpoint.trim()))?;
        // convert(&(opt.root_url.to_string() + "/sap/ui/core/designtime/api.json"))?;
    }

    Ok(())
}

fn convert(url:&str) -> Result<(), Box<std::error::Error>>{
    println!("{:?}", url);
    // let _resp: String = reqwest::get("https://sapui5.hana.ondemand.com/test-resources/sap/m/designtime/api.json")?.text()?;
    // let resp: serde_json::Value = reqwest::get("https://sapui5.hana.ondemand.com/test-resources/sap/m/designtime/api.json")?.json()?;
    let resp: serde_json::Value = reqwest::get(url)?.json()?;

    let mut symbols = vec![];
    for el in resp["symbols"].as_array().unwrap() {

        let json_str = serde_json::to_string_pretty(el).unwrap();
        let symb: Result<Symbol, _> = serde_json::from_str(&json_str);


        // if symb.is_err() {
        //     println!("{:?}", symb);
        // }

        // print json with line numbers on error
        if symb.is_err() {
            let text = serde_json::to_string_pretty(el).unwrap();
            let mut i = 0;
            for line in text.split("\n") {
                i+=1;
                println!("{}\t{}", i, line);
            }
        }

        let symbol = symb?;

        // println!("{:?}", symbol.name);
        // println!("{:?}", symbol.kind);
        // println!("{:?}", symbol.basename);
        // println!("{:?}", symbol.resource);
        // println!("{:?}", symbol.module);
        // println!("");

        // if serde_json::to_string_pretty(&symbol).unwrap().contains("JSONModel") {
        //     println!("{}", json_str);
        //     let symbs = extract_type_defs(&[symbol.clone()]);
        //     println!("{:?}", symbs[0]);

        //     fs::create_dir_all(&symbs[0].0.path)?;
        //     let mut file = File::create(&symbs[0].0.full_path())?;
        //     file.write_all(&symbs[0].1.as_bytes())?;
        // }

        symbols.push(symbol);
        
    }

    let classes = extract_type_defs(&symbols);

    for el in classes {
        if !el.0.path.starts_with("sap") {
            panic!("{:?}", el.0.path);
        }
        fs::create_dir_all(&el.0.path)?;
        let mut file = File::create(&el.0.full_path())?;
        file.write_all(&el.1.as_bytes())?;
    }
    // let char_vec:Vec<Vec<char>> = resp.split("\n").map(|line| line.chars().collect()).collect();
    // print!("{:?}", (&char_vec[0][5800 .. 5810 + 60]).iter().cloned().collect::<String>());

    // println!("{}", serde_json::from_str::<RootInterface>(&resp).unwrap_err());

    // let resp: RootInterface = serde_json::from_str(&resp)?;
    // println!("{:?}", resp);
    Ok(())
}

fn extract_type_defs(symbols: &[Symbol]) -> Vec<(FilePath, String)> {
    symbols
        .iter()
        .filter(|symb| symb.kind == Kinds::Class)
        .map(|symbol|extract_type_def(symbol))
        .filter(|el| el.0.path.starts_with("sap"))
        .collect()
}

fn convert_ui5_type_to_ts_type(ui5_type: &str) -> &str {
    if ui5_type.trim().starts_with("sap"){
        return "any" // temp hack
    }

    match ui5_type.trim() {
        "function" => "Function",
        "Promise" => "Promise<any>",
        _=> ui5_type
    }
}
fn convert_ui5_types_to_ts_types(ui5_types: &str) -> String {
    ui5_types.split("|").map(|el|convert_ui5_type_to_ts_type(el)).collect::<Vec<&str>>().join("|")
}

fn return_val_to_ts(return_value: &Option<ReturnValue>) -> String {
    return_value.as_ref().map(|val|{
        if let Some(_type) = &val._type {
            convert_ui5_types_to_ts_types(_type)
        }else{
            "void".to_string()
        }
    }).unwrap_or_else(||"void".to_string())
}
fn param_to_ts(param: &Parameter) -> String {
    format!("{}{}:{}", param.name, if param.optional {"?"}else{""}, convert_ui5_types_to_ts_types(&param._type))
}
fn params_to_ts(params: &Option<Vec<Parameter>>) -> String {
    params.as_ref().map(|params|{
        params.iter().map(|param|{
            param_to_ts(param)
        }).collect::<Vec<String>>().join(", ")
    }).unwrap_or_else(|| "".to_string())
}

fn extract_type_def(symbol: &Symbol) -> (FilePath, String) {


    let mut steps = symbol.module.split("/").collect::<Vec<_>>();
    let file_name = steps.pop().unwrap();

    let file_path = FilePath{
        path: steps.join("/"),
        file_name: file_name.to_string() + ".d.ts"
    };
    // let path = symbol.module.to_string() + ".d.ts";

    let mut lines = vec![];

    match symbol.kind {
        Kinds::Class => {
            lines.push(format!("export = {};", symbol.basename));
            lines.push(format!("declare class {} {{", symbol.basename));

            if let Some(methods) = &symbol.methods {
                lines.extend(methods.into_iter().map(|meth|{
                    let name = meth.name.to_string();
                    let return_val = return_val_to_ts(&meth.return_value);
                    let params = params_to_ts(&meth.parameters);

                    format!("    {}({}): {}", name, params, return_val)
                }));
            }
            

            lines.push("}".to_string());
        },
        _ => {},
    }


    (file_path, lines.join("\n"))
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Aggregations {
    name: String,
    #[serde(rename = "singularName")]
    singular_name: String,
    #[serde(rename = "type")]
    _type: String,
    cardinality: String,
    visibility: String,
    // description: Option<String>,
    methods: Vec<String>,
    since: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum MultiParam {
    Param(Parameter),
    Multi(Vec<Parameter>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Constructor {
    visibility: String,
    // parameters: MultiParam, //TODO can be both
    // description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Deprecated {
    since: Option<String>,
    text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Events {
    name: String,
    visibility: String,
    // description: Option<String>,
    methods: Option<Vec<String>>,
    // parameters: Option<Vec<Parameter>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GetSource {
    name: String,
    #[serde(rename = "type")]
    _type: String,
    #[serde(default)]
    optional: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Methods {
    name: String,
    visibility: String,
    #[serde(rename = "returnValue")]
    return_value: Option<ReturnValue>,
    parameters: Option<Vec<Parameter>>,
    // description: Option<String>,
    since: Option<String>,
    #[serde(rename = "static")]
    _static: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ParameterProperties {
    #[serde(rename = "getSource")]
    get_source: Option<GetSource>,
    #[serde(rename = "getParameters")]
    get_parameters: Option<GetSource>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Parameter {
    name: String,
    #[serde(rename = "type")]
    _type: String,
    #[serde(default)]
    optional: bool,
    description: Option<String>,
    #[serde(rename = "parameterProperties")]
    parameter_properties: Option<ParameterProperties>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Properties {
    name: String,
    visibility: String,
    #[serde(rename = "static")]
    _static: Option<bool>,
    #[serde(rename = "type")]
    _type: String,
    // description: Option<String>,
    group: Option<String>,
    default_value: Option<bool>,
    methods: Option<Vec<String>>,
    since: Option<String>,
}

// #[derive(Serialize, Deserialize, Debug, Clone)]
// enum Types {
//     function,
//     Promise,
//     string,
//     jQuery,
// }

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ReturnValue {
    #[serde(rename = "type")]
    _type: Option<String>,
    // _type: Option<Types>,
    // description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct RootInterface {
    #[serde(rename = "$schema-ref")]
    schema_ref: String,
    version: String,
    library: String,
    symbols: Vec<Symbol>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
enum Kinds {
    #[serde(rename = "namespace")]
    Namespace,
    #[serde(rename = "class")]
    Class,
    #[serde(rename = "enum")]
    Enum,
    #[serde(rename = "interface")]
    Interface,
    #[serde(rename = "function")]
    Function,
    #[serde(rename = "typedef")]
    Typedef,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Symbol {
    kind: Kinds,
    name: String,
    basename: String,
    resource: String,
    module: String,
    export: Option<String>,
    #[serde(rename = "static")]
    _static: Option<bool>,
    visibility: String,
    // description: Option<String>,
    properties: Option<Vec<Properties>>,
    methods: Option<Vec<Methods>>,
    extends: Option<String>,
    #[serde(rename = "ui5-metamodel")]
    ui_5_metamodel: Option<bool>,
    #[serde(rename = "ui5-metadata")]
    ui_5_metadata: Option<Ui5Metadata>,
    constructor: Option<Constructor>,
    since: Option<String>,
    events: Option<Vec<Events>>,
    implements: Option<Vec<String>>,
    #[serde(rename = "abstract")]
    _abstract: Option<bool>,
    deprecated: Option<Deprecated>,
    experimental: Option<Deprecated>,
    references: Option<Vec<String>>,
    #[serde(rename = "final")]
    _final: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Ui5Metadata {
    stereotype: Option<String>,
    properties: Option<Vec<Properties>>,
    aggregations: Option<Vec<Aggregations>>,
    #[serde(rename = "defaultAggregation")]
    default_aggregation: Option<String>,
    associations: Option<Vec<Aggregations>>,
    events: Option<Vec<Events>>,
}

