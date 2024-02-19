use log::debug;
use sui_sdk::rpc_types::SuiExecutionResult;

pub fn parse_result_u64(sui_execution_result: &SuiExecutionResult, offset: usize) -> Vec<u64>{
    let mut results = vec![];
    for (bytes, _) in sui_execution_result.return_values.iter() {
        debug!("{:?}", bytes);
        let b = &bytes[offset..(offset+8)];
        let r = u64::from_le_bytes(b.try_into().unwrap());
        results.push(r);
    }
    return results;
}