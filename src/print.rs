use holochain::conductor::*;
use holochain::sweettest::*;
use holochain_state::source_chain::*;
use holochain_zome_types::*;
use holochain_p2p::*;
use colored::*;
use crate::get_entry_names;


///
pub async fn get_dna_entry_names(conductor: &SweetConductor, cell: &SweetCell) -> Vec<Vec<String>> {
   let first_dna_hash = conductor.handle().list_dnas()[0].clone();
   let dna = conductor.handle().get_dna(&first_dna_hash).unwrap().clone();
   let mut all_entry_names = Vec::new();
   for (zome_name, _zome_def) in dna.dna_def().zomes.iter() {
      let entry_names = get_zome_entry_names(&conductor, &cell, &zome_name.0).await;
      all_entry_names.push(entry_names);
   }
   all_entry_names
}

///
pub async fn get_zome_entry_names(conductor: &SweetConductor, cell: &SweetCell, zome_name: &str) -> Vec<String> {
   let mut entry_names = Vec::new();
   let entry_defs: EntryDefsCallbackResult = conductor.call(&cell.zome(zome_name), "entry_defs", ()).await;
   let EntryDefsCallbackResult::Defs(defs) = entry_defs;
   for entry_def in defs.clone() {
      //println!("entry_def: {:?}", entry_def);
      let name = match entry_def.id {
         EntryDefId::App(name) => name,
         EntryDefId::CapClaim => "CapClaim".to_string(),
         EntryDefId::CapGrant => "CapGrant".to_string(),
      };
      entry_names.push(name);
   }
   entry_names
}


///
fn print_element(element: &SourceChainJsonElement) -> String {
   let mut str = format!("{:?} ", element.header.header_type());
   // let mut str = format!("({}) ", element.header_address);

   // if (element.header.header_type() == HeaderType::CreateLink) {
   //    str += &format!(" '{:?}'", element.header.tag());
   // }

   let entry_names = get_entry_names();

   match &element.header {
      Header::CreateLink(create_link) => {
         // let s = std::str::from_utf8(&create_link.tag.0).unwrap();
         let s = String::from_utf8_lossy(&create_link.tag.0).to_string();
         str += &format!("'{:.20}'", s).yellow().to_string();
      },
      Header::Create(create_entry) => {
         let mut s = String::new();
         match &create_entry.entry_type {
            EntryType::App(app_entry_type) => {
               s += "AppEntry ";
               let zome_index = u8::from(app_entry_type.zome_id()) as usize;
               let entry_index = u8::from(app_entry_type.id()) as usize;
               let entry_name = entry_names[zome_index][entry_index].clone();
               s += &format!("'{}'", entry_name);
               //s += &format!("z{} e{}", u8::from(app_entry_type.zome_id()), u8::from(app_entry_type.id()));
               if app_entry_type.visibility() == &EntryVisibility::Public {
                  s = s.green().to_string();
               } else {
                  s = s.red().to_string();
               }
            },
            _ => {
               s += &format!("{:?}", create_entry.entry_type);
               s = s.green().to_string();
            }
         };
         str += &s;
      },
      Header::Update(update_entry) => {
         let mut s = String::new();
         match &update_entry.entry_type {
            EntryType::App(app_entry_type) => {
               s += "AppEntry ";
               let zome_index = u8::from(app_entry_type.zome_id()) as usize;
               let entry_index = u8::from(app_entry_type.id()) as usize;
               let entry_name = entry_names[zome_index][entry_index].clone();
               s += &format!("'{}'", entry_name);
               //s += &format!("z{} e{}", u8::from(app_entry_type.zome_id()), u8::from(app_entry_type.id()));
            },
            _ => {
               s += &format!("{:?}", update_entry.entry_type);
            }
         };
         str += &s.yellow().to_string();
      },
      Header::DeleteLink(delete_link) => {
         let s = format!("{}", delete_link.link_add_address);
         str += &format!("'{:.25}'", s).yellow().to_string();
      },
      Header::Delete(delete_entry) => {
         let s = format!("{}", delete_entry.deletes_address);
         str += &format!("'{:.25}'", s).green().to_string();
      }
      _ => {},
   }
   let mut line = format!("{:<40} ({}) ({:?})", str, element.header_address, element.header.entry_hash());
   if element.header.is_genesis() {
      line = line.blue().to_string();
   }
   line
}


///
pub async fn print_chain(
   conductor: &SweetConductor,
   cell: &SweetCell,
) {
   let cell_id = cell.cell_id();
   let vault = conductor.get_authored_env(cell_id.dna_hash()).unwrap();

   let space = cell_id.dna_hash().to_kitsune();

   let env = conductor.get_p2p_env(space);
   let _peer_dump = p2p_agent_store::dump_state(
      env.into(),
      Some(cell_id.clone()),
   ).await.expect("p2p_store should not fail");

   let json_dump = dump_state(vault.clone().into(), cell.agent_pubkey().clone()).await.unwrap();
   //let json = serde_json::to_string_pretty(&json_dump).unwrap();

   if json_dump.elements.is_empty() {
      println!("\n\n>>>>>> SOURCE-CHAIN EMPTY <<<<<<\n\n");
      return;
   }

   let author = json_dump.elements[0].header.author().clone();
   println!("\n====== SOURCE-CHAIN STATE DUMP START ===== {}", author);
   //println!("source_chain_dump({}) of {:?}", json_dump.elements.len(), cell.agent_pubkey());

   let mut count = 0;
   for element in &json_dump.elements {
      let str = print_element(&element);
      println!(" {:2}. {}", count, str);
      count += 1;
   }

   println!("====== SOURCE-CHAIN STATE DUMP END  ===== {} / {}\n",
            json_dump.elements.len(), json_dump.published_ops_count);
}


///
pub async fn print_peers(conductor: &SweetConductor, cell: &SweetCell) {
   let cell_id = cell.cell_id();
   let space = cell_id.dna_hash().to_kitsune();
   let env = conductor.get_p2p_env(space);
   let peer_dump = p2p_agent_store::dump_state(
      env.into(),
      Some(cell_id.clone()),
   ).await.expect("p2p_store should not fail");
   println!(" *** peer_dump: {}\n\n",peer_dump.peers.len());
}