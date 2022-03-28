use holochain::sweettest::{SweetCell, SweetConductor};

use std::sync::Arc;
use holochain_zome_types::AppSignal;
use stream_cancel::{Trigger, Valve};

use tokio::sync::Mutex;
use tokio::task::JoinHandle;

use holochain_types::prelude::*;

use crate::print_chain;

pub struct SweeterCell {
   cell: SweetCell,
   conductor: SweetConductor,
   /// Signal handling
   app_signals: Arc<Mutex<Vec<AppSignal>>>,
   #[allow(dead_code)]
   jh: JoinHandle<()>,
   #[allow(dead_code)]
   trigger: Trigger,
}


// impl Drop for SweeterCell {
//    fn drop(&mut self) {
//       self.trigger.cancel();
//       self.jh.await.unwrap();
//    }
// }


impl SweeterCell {
   ///
   pub async fn new(mut conductor: SweetConductor, cell: SweetCell) -> Self {
      /// Create thread for receiving signals
      let app_signals = Arc::new(Mutex::new(vec![]));
      let (trigger, valve) = Valve::new();
      use futures::stream::StreamExt;
      let mut stream = valve.wrap(conductor.signals());
      let jh = tokio::task::spawn({
         let clone = Arc::clone(&app_signals);
         async move {
            while let Some(Signal::App(_, app_signal)) = stream.next().await {
               //let signal: SignalProtocol = app_signal.into_inner().decode().unwrap();
               //println!("\n SIGNAL RECEIVED: {:?}\n\n", signal);
               let mut v = clone.lock().await;
               v.push(app_signal);
            }
         }
      });
      /// Done
      Self {
         cell,
         conductor,
         app_signals,
         jh,
         trigger,
      }
   }



   pub fn key(&self) -> AgentPubKey {
      self.cell.agent_pubkey().clone()
   }

   pub fn dna_hash(&self) -> DnaHash {
      self.cell.dna_hash().clone()
   }


   pub async fn drain_signals(&mut self) -> Vec<AppSignal> {
      self.app_signals.lock().await.drain(..).collect()
   }

   pub async fn drain_signals_test(&mut self) {
      println!("Before drain count: {}", self.app_signals.lock().await.len());
      let _signals = self.drain_signals().await;
      println!("After drain count: {}", self.app_signals.lock().await.len());
   }

   // pub async fn print_signals(&self) {
   //    println!("\n****** SIGNALS DUMP START ****** {}", self.key());
   //    let signals = self.signals.lock().await.clone();
   //    let mut count = 0;
   //    for signal in signals {
   //       println!(" {:2}. {:?}", count, signal);
   //       count += 1;
   //    }
   //    println!("\n****** SIGNALS DUMP END   ****** {}", count);
   // }

   ///
   pub async fn print_chain(&self) {
      print_chain(&self.conductor, &self.cell).await;
   }

   ///
   pub async fn call_any_zome<I, O>(&self, zome_name: &str, fn_name: &str, payload: I) -> O
      where
         I: serde::Serialize + std::fmt::Debug,
         O: serde::de::DeserializeOwned + std::fmt::Debug,
   {
      return self.conductor.call(&self.cell.zome(zome_name), fn_name, payload).await;
   }


   /// Tries 10 times to get a certain result from a zome call
   /// Returns Err on failure to get the expected result
   pub async fn try_call_zome<P, T>(
      &self,
      zome_name: &str,
      fn_name: &str,
      payload: P,
      predicat: fn(res: &T) -> bool,
   ) -> Result<T, ()>
      where
         T: serde::de::DeserializeOwned + std::fmt::Debug,
         P: Clone + serde::Serialize + std::fmt::Debug,
   {
      for _ in 0..10u32 {
         let res: T = self.conductor.call(&self.cell.zome(zome_name), fn_name, payload.clone())
                          .await;
         if predicat(&res) {
            return Ok(res);
         }
         tokio::time::sleep(std::time::Duration::from_millis(2 * 1000)).await;
      }
      Err(())
   }
}