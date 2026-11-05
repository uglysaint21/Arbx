use reqwest::{redirect::Policy,Client};
use serde::Serialize;
use serde_json::{json,Value};
use std::{net::IpAddr,time::Duration};
use zeroize::Zeroizing;
use tauri::State;

const RPC_SERVICE:&str="com.arbx.desktop";
const RPC_USER:&str="polygon-mainnet-rpc";
const POLYGON_CHAIN_ID:u64=137;
const MAX_RESPONSE_BYTES:usize=64*1024;

#[derive(Serialize)]
pub struct RpcState{configured:bool}
#[derive(Serialize)]
pub struct RpcHealth{connected:bool,chain_id:u64,block_number:u64}

pub fn validate_endpoint(input:&str)->Result<url::Url,String>{
    if input.len()>8192||input.trim()!=input||input.is_empty(){return Err("Enter a valid RPC endpoint.".into())}
    let parsed=url::Url::parse(input).map_err(|_|"The RPC endpoint must be a valid URL.".to_string())?;
    let authority=input.split_once("://").map(|(_,rest)|rest.split(['/', '?', '#']).next().unwrap_or("")).unwrap_or("");
    if authority.contains('@')||!parsed.username().is_empty()||parsed.password().is_some()||parsed.fragment().is_some(){return Err("RPC endpoint URLs must not contain embedded user information or fragments.".into())}
    let host=parsed.host_str().ok_or_else(||"The RPC endpoint must include a host.".to_string())?;
    match parsed.scheme(){
        "https"=>{},
        "http"=>{
            let ip_host=host.strip_prefix('[').and_then(|value|value.strip_suffix(']')).unwrap_or(host);
            let loopback=host.eq_ignore_ascii_case("localhost")||ip_host.parse::<IpAddr>().map(|ip|ip.is_loopback()).unwrap_or(false);
            if !loopback{return Err("Use HTTPS for remote RPC providers. HTTP is allowed only for localhost or a loopback IP.".into())}
        },
        _=>return Err("RPC endpoints must use HTTPS (or HTTP on loopback only).".into())
    }
    Ok(parsed)
}

#[cfg(windows)]
fn entry()->Result<keyring::Entry,String>{keyring::Entry::new(RPC_SERVICE,RPC_USER).map_err(|_|"Windows Credential Manager is unavailable.".to_string())}
#[cfg(windows)]
pub fn configured()->Result<bool,String>{match entry()?.get_secret(){Ok(value)=>{let _secret=Zeroizing::new(value);Ok(true)},Err(keyring::Error::NoEntry)=>Ok(false),Err(_)=>Err("The saved Polygon RPC credential could not be read.".into())}}
#[cfg(not(windows))]
pub fn configured()->Result<bool,String>{Ok(false)}
#[cfg(windows)]
pub fn save_endpoint(endpoint:&str)->Result<(),String>{validate_endpoint(endpoint)?;entry()?.set_secret(endpoint.as_bytes()).map_err(|_|"Windows Credential Manager could not save the RPC endpoint.".to_string())}
#[cfg(not(windows))]
pub fn save_endpoint(_: &str)->Result<(),String>{Err("RPC endpoint storage is supported only in the Windows desktop application.".into())}
#[cfg(windows)]
fn load_endpoint()->Result<Zeroizing<String>,String>{
    let secret=Zeroizing::new(entry()?.get_secret().map_err(|_|"No Polygon RPC endpoint is configured.".to_string())?);
    let text=std::str::from_utf8(&secret).map_err(|_|"The saved Polygon RPC endpoint is invalid.".to_string())?;
    let value=Zeroizing::new(text.to_owned());
    validate_endpoint(&value)?;Ok(value)
}
#[cfg(not(windows))]
fn load_endpoint()->Result<Zeroizing<String>,String>{Err("RPC endpoint access is supported only in the Windows desktop application.".into())}
#[cfg(windows)]
pub fn clear_endpoint()->Result<(),String>{match entry()?.delete_credential(){Ok(())|Err(keyring::Error::NoEntry)=>Ok(()),Err(_)=>Err("Windows Credential Manager could not remove the RPC endpoint.".into())}}
#[cfg(not(windows))]
pub fn clear_endpoint()->Result<(),String>{Err("RPC endpoint storage is supported only in the Windows desktop application.".into())}

async fn rpc_string(client:&Client,endpoint:&str,method:&str)->Result<String,String>{
    let response=client.post(endpoint).json(&json!({"jsonrpc":"2.0","id":1,"method":method,"params":[]})).send().await.map_err(|_|"Could not reach the configured Polygon RPC provider.".to_string())?;
    if !response.status().is_success(){return Err(format!("The Polygon RPC provider returned HTTP {}.",response.status().as_u16()))}
    if response.content_length().map(|n|n>MAX_RESPONSE_BYTES as u64).unwrap_or(false){return Err("The Polygon RPC response exceeded the allowed size.".into())}
    let mut response=response;let mut body=Vec::with_capacity(1024);
    while let Some(chunk)=response.chunk().await.map_err(|_|"Could not read the Polygon RPC response.".to_string())?{
        if body.len()+chunk.len()>MAX_RESPONSE_BYTES{return Err("The Polygon RPC response exceeded the allowed size.".into())}
        body.extend_from_slice(&chunk);
    }
    let parsed:Value=serde_json::from_slice(&body).map_err(|_|"The Polygon RPC provider returned invalid JSON.".to_string())?;
    if parsed.get("jsonrpc").and_then(Value::as_str)!=Some("2.0")||parsed.get("id").and_then(Value::as_i64)!=Some(1){return Err("The Polygon RPC response did not match the JSON-RPC request.".into())}
    if parsed.get("error").is_some(){return Err("The Polygon RPC provider rejected the JSON-RPC request.".into())}
    parsed.get("result").and_then(Value::as_str).map(str::to_string).ok_or_else(||"The Polygon RPC response did not contain a valid result.".into())
}
pub async fn test_saved_endpoint()->Result<RpcHealth,String>{
    let endpoint=load_endpoint()?;
    let client=Client::builder().timeout(Duration::from_secs(10)).connect_timeout(Duration::from_secs(5)).redirect(Policy::none()).build().map_err(|_|"The Polygon RPC client could not be initialized.".to_string())?;
    let chain=parse_quantity(&rpc_string(&client,&endpoint,"eth_chainId").await?)?;
    if chain!=POLYGON_CHAIN_ID{return Err(format!("The configured RPC endpoint reported chain ID {chain}; ArbX requires Polygon PoS chain ID 137."))}
    let block=parse_quantity(&rpc_string(&client,&endpoint,"eth_blockNumber").await?)?;
    Ok(RpcHealth{connected:true,chain_id:chain,block_number:block})
}
fn parse_quantity(value:&str)->Result<u64,String>{
    let digits=value.strip_prefix("0x").ok_or_else(||"The Polygon RPC provider returned an invalid hexadecimal quantity.".to_string())?;
    if digits.is_empty()||digits.len()>16||!digits.bytes().all(|b|b.is_ascii_hexdigit()){return Err("The Polygon RPC provider returned an invalid hexadecimal quantity.".into())}
    u64::from_str_radix(digits,16).map_err(|_|"The Polygon RPC provider returned an invalid hexadecimal quantity.".into())
}

#[tauri::command]
pub fn polygon_rpc_state(state:State<'_,crate::AppState>)->Result<RpcState,String>{crate::require_authenticated(&state)?;Ok(RpcState{configured:configured()?})}
#[tauri::command]
pub async fn configure_polygon_rpc(state:State<'_,crate::AppState>)->Result<RpcState,String>{
    crate::require_authenticated(&state)?;
    let endpoint=tauri::async_runtime::spawn_blocking(crate::wallet::prompt_for_rpc_endpoint).await.map_err(|_|"The secure RPC-entry task could not be started.".to_string())??;
    crate::require_authenticated(&state)?;save_endpoint(&endpoint)?;Ok(RpcState{configured:true})
}
#[tauri::command]
pub async fn test_polygon_rpc(state:State<'_,crate::AppState>)->Result<RpcHealth,String>{crate::require_authenticated(&state)?;let result=test_saved_endpoint().await?;crate::require_authenticated(&state)?;Ok(result)}
#[tauri::command]
pub fn clear_polygon_rpc(state:State<'_,crate::AppState>)->Result<RpcState,String>{crate::require_authenticated(&state)?;clear_endpoint()?;Ok(RpcState{configured:false})}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn accepts_https_and_loopback_http(){
        assert!(validate_endpoint("https://polygon.example/rpc?key=abc").is_ok());
        assert!(validate_endpoint("http://localhost:8545").is_ok());
        assert!(validate_endpoint("http://127.0.0.1:8545").is_ok());
        assert!(validate_endpoint("http://[::1]:8545").is_ok());
    }
    #[test]fn rejects_remote_http_and_unsafe_url_parts(){
        assert!(validate_endpoint("http://rpc.example").is_err());
        assert!(validate_endpoint("https://user:pass@rpc.example").is_err());
        assert!(validate_endpoint("https://@rpc.example").is_err());
        assert!(validate_endpoint("https://rpc.example/path#fragment").is_err());
        assert!(validate_endpoint("ftp://rpc.example").is_err());
        assert!(validate_endpoint("not a url").is_err());
        assert!(validate_endpoint(" https://rpc.example").is_err());
    }
    #[test]fn quantity_parser_checks_hex_and_bounds(){
        assert_eq!(parse_quantity("0x89").unwrap(),137);assert_eq!(parse_quantity("0x0").unwrap(),0);
        assert!(parse_quantity("137").is_err());assert!(parse_quantity("0x").is_err());assert!(parse_quantity("0x10000000000000000").is_err());
    }
}