use holochain::sweettest::*;
use holochain::conductor::api::error::ConductorApiResult;


/// Call a zome function several times, waiting for a certainr result
pub async fn try_zome_call_fallible<T,P>(
   conductor: &SweetConductor,
   cell: &SweetCell,
   zome_name: &str,
   fn_name: &str,
   payload: P,
) -> Result<T, ()>
   where
      T: serde::de::DeserializeOwned + std::fmt::Debug,
      P: Clone + serde::Serialize + std::fmt::Debug,
{
   for _ in 0..10u32 {
      let maybe: ConductorApiResult<T> = conductor.call_fallible(&cell.zome(zome_name), fn_name, payload.clone())
                                                  .await;
      println!("try_zome_call_fallible() maybe = {:?}", maybe);
      if let Ok(res) = maybe {
         return Ok(res);
      }
      tokio::time::sleep(std::time::Duration::from_millis(2 * 1000)).await;
   }
   Err(())
}


/// Call a zome function several times, waiting for a certainr result
pub async fn try_zome_call<T,P>(
   conductor: &SweetConductor,
   cell: &SweetCell,
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
      let res: T = conductor.call(&cell.zome(zome_name), fn_name, payload.clone())
                            .await;
      if predicat(&res) {
         return Ok(res);
      }
      tokio::time::sleep(std::time::Duration::from_millis(2 * 1000)).await;
   }
   Err(())
}