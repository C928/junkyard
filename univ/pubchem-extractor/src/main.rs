#![allow(non_snake_case)]

use serde::{Deserialize, Deserializer, Serialize};
use std::fs;
use std::fs::File;
use std::io::{copy, Cursor, Write};
use tera::{Context, Tera};

/// https://pubchem.ncbi.nlm.nih.gov/docs/pug-rest#section=Compound-Property-Tables
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let compound_names: Vec<String> = fs::read_to_string("src/compound_names")
        .unwrap()
        .lines()
        .map(|s| s.to_string())
        .collect();

    let compounds_count = compound_names.len();
    let mut compound_number = 0;
    for compound_name in compound_names {
        println!("compound {compound_number}/{compounds_count}: {compound_name}");
        let image_available = store_compound_image(&compound_name).await;
        println!(
            "[*] image: {}\n[*] data : {}",
            if image_available { "ok" } else { "not found" },
            if store_compound_properties(&compound_name, image_available).await {
                "ok"
            } else {
                "not found"
            }
        );
        compound_number += 1;
    }

    Ok(())
}

async fn store_compound_image(compound_name: &str) -> bool {
    let url =
        format!("https://pubchem.ncbi.nlm.nih.gov/rest/pug/compound/name/{compound_name}/PNG");

    let resp = reqwest::get(url).await.unwrap();
    if resp.status().is_success() {
        let mut image_file = File::create(format!("output/img/{compound_name}.png")).unwrap();
        let mut content = Cursor::new(resp.bytes().await.unwrap());
        copy(&mut content, &mut image_file).unwrap();
        return true;
    }

    false
}

#[derive(Debug, Deserialize)]
struct PropertyTable {
    PropertyTable: Properties,
}

#[derive(Debug, Deserialize)]
struct Properties {
    Properties: Vec<Property>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Property {
    CID: u32,
    #[serde(deserialize_with = "parse_f32")]
    MolecularWeight: f32,
    CanonicalSMILES: String,
    IsomericSMILES: String,
    InChI: String,
    InChIKey: String,
    IUPACName: String,
    #[serde(deserialize_with = "parse_f32")]
    ExactMass: f32,
    #[serde(deserialize_with = "parse_f32")]
    MonoisotopicMass: f32,
    TPSA: f32,
    Complexity: u16,
    Charge: u8,
    HBondDonorCount: u8,
    HBondAcceptorCount: u8,
    RotatableBondCount: u8,
    HeavyAtomCount: u8,
    IsotopeAtomCount: u8,
    AtomStereoCount: u8,
    DefinedAtomStereoCount: u8,
    UndefinedAtomStereoCount: u8,
    BondStereoCount: u8,
    DefinedBondStereoCount: u8,
    UndefinedBondStereoCount: u8,
    CovalentUnitCount: u8,
    LiteratureCount: u32,
}

fn parse_f32<'de, D>(deserializer: D) -> Result<f32, D::Error>
    where
        D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    s.parse::<f32>().map_err(serde::de::Error::custom)
}

const REQUESTED_PROPERTIES: &str = "MolecularFormula,MolecularWeight,CanonicalSMILES,\
IsomericSMILES,InChI,InChIKey,IUPACName,ExactMass,MonoisotopicMass,TPSA,Complexity,Charge,\
HBondDonorCount,HBondAcceptorCount,RotatableBondCount,HeavyAtomCount,IsotopeAtomCount,\
AtomStereoCount,DefinedAtomStereoCount,UndefinedAtomStereoCount,BondStereoCount,\
DefinedBondStereoCount,UndefinedBondStereoCount,CovalentUnitCount,LiteratureCount";

async fn store_compound_properties(
    compound_name: &str,
    image_available: bool,
) -> bool {
    let mut context = Context::new();
    context.insert("compound_name", compound_name);
    context.insert("image_available", &image_available);
    context.insert("image_src", &format!("img/{compound_name}.png"));

    let mut tera = Tera::default();
    let template_path = "src/compound-template.html";
    tera.add_template_file(template_path, Some("compound-template"))
        .unwrap();

    let url = format!(
        "https://pubchem.ncbi.nlm.nih.gov/rest/pug/compound/name/{compound_name}/property/\
        {REQUESTED_PROPERTIES}/JSON"
    );
    let mut data_found = true;
    let resp = reqwest::get(url).await.unwrap();
    if resp.status().is_success() {
        let property_table = resp.json::<PropertyTable>().await.unwrap();
        let property = &property_table.PropertyTable.Properties[0];
        context.insert("properties", &property);
    } else {
        data_found = false;
    }

    let rendered_html = tera.render("compound-template", &context).unwrap();
    let mut file = File::create(format!("output/{compound_name}.html")).unwrap();
    file.write_all(rendered_html.as_bytes()).unwrap();
    data_found
}
