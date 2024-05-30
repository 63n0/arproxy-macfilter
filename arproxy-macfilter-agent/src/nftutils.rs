use std::{fs::File, io::BufReader, path::PathBuf};

use nftables::{
    self,
    batch::Batch,
    expr::Expression,
    helper,
    schema::{self, Element, Nftables},
    types::{NfFamily},
};
use pnet::util::MacAddr;

pub fn add_mac_element(
    family: NfFamily,
    table: &String,
    set: &String,
    addr: &MacAddr,
) -> Result<(), anyhow::Error> {
    let mut batch = Batch::new();
    batch.add(schema::NfListObject::Element(Element {
        family: family,
        table: table.clone(),
        name: set.clone(),
        elem: vec![Expression::String(addr.to_string())],
    }));

    let nftobj = batch.to_nftables();
    helper::apply_ruleset(&nftobj, None, None);
    Ok(())
}

pub fn apply_rulesets_from_file(path: &PathBuf) -> Result<(), anyhow::Error> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let nftobj: Nftables = serde_json::from_reader(reader)?;

    helper::apply_ruleset(&nftobj, None, None);
    Ok(())
}
