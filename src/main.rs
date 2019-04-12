#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate structopt;



#[derive(StructOpt, Debug)]
#[structopt(name = "upload")]
struct Opt {
    /// The number of documents per elastic call
    file_in: String,
}

fn main() -> Result<(), Box<std::error::Error>>{
        println!("Hello, world!");

        // let _resp: String = reqwest::get("https://sapui5.hana.ondemand.com/test-resources/sap/m/designtime/api.json")?.text()?;
        let resp: serde_json::Value = reqwest::get("https://sapui5.hana.ondemand.com/test-resources/sap/m/designtime/api.json")?.json()?;
        // resp["symbols"].as_array().map(|el|{
        //     let symb:Symbol = serde_json::from_str(el.to_string())?;
        // })

        let mut symbols = vec![];
        for el in resp["symbols"].as_array().unwrap() {
            let symb: Result<Symbol, _> = serde_json::from_str(&serde_json::to_string_pretty(el).unwrap());
            if symb.is_err() {
                let text = serde_json::to_string_pretty(el).unwrap();
                let mut i = 0;
                for line in text.split("\n") {
                    i+=1;
                    println!("{}\t{}", i, line);
                }
            }
            symbols.push(symb?)
        }

        // let char_vec:Vec<Vec<char>> = resp.split("\n").map(|line| line.chars().collect()).collect();
        // print!("{:?}", (&char_vec[0][5800 .. 5810 + 60]).iter().cloned().collect::<String>());

        // println!("{}", serde_json::from_str::<RootInterface>(&resp).unwrap_err());

        // let resp: RootInterface = serde_json::from_str(&resp)?;
        // println!("{:?}", resp);
        Ok(())

}

#[derive(Serialize, Deserialize, Debug)]
struct Aggregations {
    name: String,
    #[serde(rename = "singularName")]
    singular_name: String,
    #[serde(rename = "type")]
    _type: String,
    cardinality: String,
    visibility: String,
    description: Option<String>,
    methods: Vec<String>,
    since: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
enum MultiParam {
    Param(Parameters),
    Multi(Vec<Parameters>),
}

#[derive(Serialize, Deserialize, Debug)]
struct Constructor {
    visibility: String,
    // parameters: MultiParam, //TODO can be both
    description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Deprecated {
    since: String,
    text: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Events {
    name: String,
    visibility: String,
    description: Option<String>,
    methods: Option<Vec<String>>,
    // parameters: Option<Vec<Parameters>>,
}

#[derive(Serialize, Deserialize, Debug)]
struct GetSource {
    name: String,
    #[serde(rename = "type")]
    _type: String,
    #[serde(default)]
    optional: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct Methods {
    name: String,
    visibility: String,
    #[serde(rename = "returnValue")]
    return_value: Option<ReturnValue>,
    parameters: Option<Vec<Parameters>>,
    description: Option<String>,
    since: Option<String>,
    #[serde(rename = "static")]
    _static: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ParameterProperties {
    #[serde(rename = "getSource")]
    get_source: GetSource,
    #[serde(rename = "getParameters")]
    get_parameters: GetSource,
}

#[derive(Serialize, Deserialize, Debug)]
struct Parameters {
    name: String,
    // #[serde(rename = "type")]
    // _type: String,
    // #[serde(default)]
    // optional: bool,
    // description: Option<String>,
    // #[serde(rename = "parameterProperties")]
    // parameter_properties: Option<ParameterProperties>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Properties {
    name: String,
    visibility: String,
    #[serde(rename = "static")]
    _static: Option<bool>,
    #[serde(rename = "type")]
    _type: String,
    description: Option<String>,
    group: Option<String>,
    default_value: Option<bool>,
    methods: Option<Vec<String>>,
    since: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ReturnValue {
    #[serde(rename = "type")]
    _type: Option<String>,
    description: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct RootInterface {
    #[serde(rename = "$schema-ref")]
    schema_ref: String,
    version: String,
    library: String,
    symbols: Vec<Symbol>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Symbol {
    kind: String,
    name: String,
    basename: String,
    resource: String,
    module: String,
    export: Option<String>,
    #[serde(rename = "static")]
    _static: bool,
    visibility: String,
    description: Option<String>,
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

#[derive(Serialize, Deserialize, Debug)]
struct Ui5Metadata {
    stereotype: String,
    properties: Option<Vec<Properties>>,
    aggregations: Option<Vec<Aggregations>>,
    #[serde(rename = "defaultAggregation")]
    default_aggregation: Option<String>,
    associations: Option<Vec<Aggregations>>,
    events: Option<Vec<Events>>,
}

