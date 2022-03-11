use holochain::sweettest::*;
use holochain::conductor::config::ConductorConfig;
use holo_hash::*;
use futures::future;
use std::sync::Mutex;
use once_cell::sync::Lazy;

use crate::*;


static g_entry_names: Lazy<Mutex<Vec<Vec<String>>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub fn get_entry_names() -> Vec<Vec<String>> {
   g_entry_names.lock().unwrap().clone()
}

pub fn set_entry_names(entry_names: Vec<Vec<String>>) {
   *g_entry_names.lock().unwrap() = entry_names;
}

///
pub fn create_network_config() -> ConductorConfig {
   std::env::set_var("KIT_PROXY", "kitsune-proxy://SYVd4CF3BdJ4DS7KwLLgeU3_DbHoZ34Y-qroZ79DOs8/kitsune-quic/h/165.22.32.11/p/5779/--");
   let kitsune_config = SweetNetwork::env_var_proxy()
      .expect("KIT_PROXY not set");
   let mut config = ConductorConfig::default();
   config.network = Some(kitsune_config);
   config
}



///
pub async fn setup_conductors(dna_filepath: &str, n: usize) -> (SweetConductorBatch, Vec<AgentPubKey>, SweetAppBatch) {
   let dna = SweetDnaFile::from_bundle(std::path::Path::new(dna_filepath))
      .await
      .unwrap();

   // let mut network = SweetNetwork::env_var_proxy().unwrap_or_else(|| {
   //    println!("KIT_PROXY not set using local quic network");
   //    SweetNetwork::local_quic()
   // });
   // let mut network = SweetNetwork::local_quic();
   // network.network_type = kitsune_p2p::NetworkType::QuicMdns;
   // let mut config = holochain::conductor::config::ConductorConfig::default();
   // config.network = Some(network);

   // /// Common config with proxy
   //let config = create_network_config();
   //let mut conductors = SweetConductorBatch::from_config(n, config).await;

   /// Default config
   let mut conductors = SweetConductorBatch::from_standard_config(n).await;

   let all_agents: Vec<AgentPubKey> =
      future::join_all(conductors.iter().map(|c| SweetAgents::one(c.keystore()))).await;
   println!("\n* INSTALLING APP...");
   let apps = conductors
      .setup_app_for_zipped_agents("app", &all_agents, &[dna])
      .await
      .unwrap();

   println!("\n* RETRIEVING ENTRY NAMES...");
   let cell1 = apps.iter().next().unwrap().clone().into_cells()[0].clone();
   let all_entry_names = get_dna_entry_names(&conductors[0], &cell1).await;
   set_entry_names(all_entry_names);

   println!("\n* EXCHANGING PEER INFO...");
   conductors.exchange_peer_info().await;
   println!("\n* CONDUCTORS SETUP DONE\n\n");
   (conductors, all_agents, apps)
}


///
pub async fn setup_1_conductor(dna_filepath: &str) -> (SweetConductor, AgentPubKey, SweetCell) {
   let dna = SweetDnaFile::from_bundle(std::path::Path::new(dna_filepath))
      .await
      .unwrap();

   /// QuicMdns Config
   // let mut network = SweetNetwork::local_quic();
   // network.network_type = kitsune_p2p::NetworkType::QuicMdns;
   // let mut config = holochain::conductor::config::ConductorConfig::default();
   // config.network = Some(network);
   // let mut conductor = SweetConductor::from_config(config).await;

   /// Standard config
   let mut conductor = SweetConductor::from_standard_config().await;

   let alex = SweetAgents::one(conductor.keystore()).await;
   let app1 = conductor
      .setup_app_for_agent("app", alex.clone(), &[dna.clone()])
      .await
      .unwrap();

   let cell1 = app1.into_cells()[0].clone();

   let all_entry_names = get_dna_entry_names(&conductor, &cell1).await;
   set_entry_names(all_entry_names);

   println!("\n\n\n SETUP DONE\n\n");

   (conductor, alex, cell1)
}
