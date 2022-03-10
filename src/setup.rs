use holochain::sweettest::*;
use holochain_zome_types::*;
use holochain::conductor::config::ConductorConfig;
use holo_hash::*;
use futures::future;
use tokio::time::{sleep, Duration};

use crate::*;


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
   println!("\n* EXCHANGING PEER INFO...");
   conductors.exchange_peer_info().await;
   println!("\n* CONDUCTORS SETUP DONE\n\n");
   (conductors, all_agents, apps)
}


///
pub async fn setup_1_conductor(dna_filepath: &str) -> (SweetConductor, AgentPubKey, SweetCell, Vec<Vec<String>>) {
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


   println!("\n\n\n SETUP DONE\n\n");

   (conductor, alex, cell1, all_entry_names)
}


///
pub async fn setup_2_conductors(dna_filepath: &str) -> (SweetConductorBatch, Vec<AgentPubKey>, SweetAppBatch) {
   let (conductors, agents, apps) = setup_conductors(dna_filepath, 2).await;
   let cells = apps.cells_flattened();

   println!("* WAITING FOR INIT TO FINISH...\n\n");
   sleep(Duration::from_millis(5 * 1000)).await;

   println!("\n\n\n CALLING get_enc_key() TO SELF ...\n\n");
   let _: X25519PubKey = try_zome_call_fallible(&conductors[0], &cells[0],"delivery", "get_enc_key", &agents[0])
      .await.unwrap();
   let _: X25519PubKey = try_zome_call_fallible(&conductors[1], &cells[1],"delivery", "get_enc_key", &agents[1])
      .await.unwrap();
   println!("\n\n\n CALLING get_enc_key() TO FRIEND ...\n\n");
   let _: X25519PubKey = try_zome_call_fallible(&conductors[1], &cells[1],"delivery", "get_enc_key", &agents[0])
      .await.unwrap();
   println!("\n\n\n AGENTS SETUP DONE\n\n");

   print_peers(&conductors[1], &cells[1]).await;

   (conductors, agents, apps)
}


///
pub async fn setup_3_conductors(dna_filepath: &str) -> (SweetConductorBatch, Vec<AgentPubKey>, SweetAppBatch) {
   let (conductors, agents, apps) = setup_conductors(dna_filepath, 3).await;
   let cells = apps.cells_flattened();

   println!("\n\n\n WAITING FOR INIT TO FINISH...\n\n");
   sleep(Duration::from_millis(10 * 1000)).await;

   let _: X25519PubKey = try_zome_call_fallible(&conductors[0], &cells[0],"delivery", "get_enc_key", &agents[0])
      .await.unwrap();
   let _: X25519PubKey = try_zome_call_fallible(&conductors[1], &cells[1],"delivery", "get_enc_key", &agents[1])
      .await.unwrap();
   //let _: X25519PubKey = conductors[1].call(&cells[1].zome("delivery"), "get_enc_key", &agents[1]).await;

   println!("\n\n\n AGENTS SETUP DONE\n\n");

   (conductors, agents, apps)
}
