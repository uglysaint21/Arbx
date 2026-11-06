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

async fn rpc_string_params(client:&Client,endpoint:&str,method:&str,params:Value)->Result<String,String>{
    let response=client.post(endpoint).json(&json!({"jsonrpc":"2.0","id":1,"method":method,"params":params})).send().await.map_err(|_|"Could not reach the configured Polygon RPC provider.".to_string())?;
    if !response.status().is_success(){return Err(format!("The Polygon RPC provider returned HTTP {}.",response.status().as_u16()))}
    if response.content_length().map(|n|n>MAX_RESPONSE_BYTES as u64).unwrap_or(false){return Err("The Polygon RPC response exceeded the allowed size.".into())}
    let mut response=response;let mut body=Vec::with_capacity(1024);
    while let Some(chunk)=response.chunk().await.map_err(|_|"Could not read the Polygon RPC response.".to_string())?{
        if body.len()+chunk.len()>MAX_RESPONSE_BYTES{return Err("The Polygon RPC response exceeded the allowed size.".into())}
        body.extend_from_slice(&chunk);
    }
    parse_rpc_result(&body)
}
async fn rpc_string(client:&Client,endpoint:&str,method:&str)->Result<String,String>{
    rpc_string_params(client,endpoint,method,json!([])).await
}
fn parse_rpc_result(body:&[u8])->Result<String,String>{
    let parsed:Value=serde_json::from_slice(body).map_err(|_|"The Polygon RPC provider returned invalid JSON.".to_string())?;
    if parsed.get("jsonrpc").and_then(Value::as_str)!=Some("2.0")||parsed.get("id").and_then(Value::as_i64)!=Some(1){return Err("The Polygon RPC response did not match the JSON-RPC request.".into())}
    if parsed.get("error").filter(|error|!error.is_null()).is_some(){return Err("The Polygon RPC provider rejected the JSON-RPC request.".into())}
    parsed.get("result").and_then(Value::as_str).map(str::to_string).ok_or_else(||"The Polygon RPC response did not contain a valid result.".into())
}

#[derive(Serialize)]
pub struct TokenBalance { address:String, symbol:String, decimals:u8, balance:String }
#[derive(Serialize)]
pub struct WalletBalances { address:String, native_balance:String, tokens:Vec<TokenBalance> }

fn valid_evm_address(value:&str)->bool {
    value.len()==42 && value.starts_with("0x") && value[2..].bytes().all(|b|b.is_ascii_hexdigit())
}
fn parse_big_quantity(value:&str)->Result<String,String>{
    let digits=value.strip_prefix("0x").ok_or_else(||"The RPC provider returned an invalid hexadecimal balance.".to_string())?;
    if digits.is_empty() || digits.len()>64 || !digits.bytes().all(|b|b.is_ascii_hexdigit()) {
        return Err("The RPC provider returned an invalid hexadecimal balance.".into());
    }
    let mut decimal=String::from("0");
    for byte in digits.bytes() {
        let digit=(byte as char).to_digit(16).ok_or_else(||"The RPC provider returned an invalid hexadecimal balance.".to_string())? as u32;
        let mut carry=digit;
        let mut chars:Vec<u8>=decimal.bytes().collect();
        for c in chars.iter_mut().rev() {
            let value=((*c-b'0') as u32)*16+carry;
            *c=b'0'+(value%10) as u8;
            carry=value/10;
        }
        while carry>0 { chars.insert(0,b'0'+(carry%10) as u8); carry/=10; }
        decimal=String::from_utf8(chars).map_err(|_|"Could not decode the token balance.".to_string())?;
    }
    Ok(decimal)
}
fn format_units(raw:&str,decimals:u8)->String{
    let decimals=decimals as usize;
    if decimals==0{return raw.to_string()}
    let padded=if raw.len()<=decimals { format!("{}{}","0".repeat(decimals+1-raw.len()),raw) } else { raw.to_string() };
    let split=padded.len()-decimals;
    let whole=&padded[..split];
    let fraction=padded[split..].trim_end_matches('0');
    if fraction.is_empty(){whole.to_string()}else{format!("{whole}.{fraction}")}
}
fn decode_abi_string(value:&str)->Result<String,String>{
    let data=value.strip_prefix("0x").ok_or_else(||"The token returned invalid symbol data.".to_string())?;
    if data.len()<64 || data.len()%2!=0 || !data.bytes().all(|b|b.is_ascii_hexdigit()) {
        return Err("The token returned invalid symbol data.".into());
    }
    let offset=usize::from_str_radix(&data[..64],16).unwrap_or(usize::MAX);
    let (start,len)=if offset==32 && data.len()>=128 {
        let len=usize::from_str_radix(&data[64..128],16).map_err(|_|"The token returned invalid symbol data.".to_string())?;
        (128,len)
    } else {(0,32)};
    if len==0 || len>64 || data.len()<start+len*2{return Err("The token returned an unsupported symbol value.".into())}
    let bytes=(0..len).map(|i|u8::from_str_radix(&data[start+i*2..start+i*2+2],16).unwrap_or(0)).collect::<Vec<_>>();
    let symbol=String::from_utf8(bytes).map_err(|_|"The token symbol is not valid UTF-8.".to_string())?;
    let symbol=symbol.trim_matches(char::from(0)).trim().to_string();
    if symbol.is_empty() || symbol.chars().any(char::is_control){return Err("The token returned an invalid symbol.".into())}
    Ok(symbol)
}
async fn eth_call(client:&Client,endpoint:&str,to:&str,data:String)->Result<String,String>{
    rpc_string_params(client,endpoint,"eth_call",json!([{"to":to,"data":data},"latest"])).await
}
pub async fn read_saved_wallet_balances(address:String,token_addresses:Vec<String>)->Result<WalletBalances,String>{
    if !valid_evm_address(&address){return Err("The saved wallet address is invalid. Re-import the wallet before continuing.".into())}
    if token_addresses.len()>10{return Err("Read up to 10 token contracts at a time.".into())}
    for token in &token_addresses {
        if !valid_evm_address(token){return Err("Every token contract must be a valid 20-byte EVM address.".into())}
    }
    let endpoint=load_endpoint()?;
    let client=Client::builder().timeout(Duration::from_secs(12)).connect_timeout(Duration::from_secs(5)).redirect(Policy::none()).build().map_err(|_|"The Polygon RPC client could not be initialized.".to_string())?;
    let chain=parse_quantity(&rpc_string(&client,&endpoint,"eth_chainId").await?)?;
    if chain!=POLYGON_CHAIN_ID{return Err(format!("The configured RPC endpoint reported chain ID {chain}; ArbX requires Polygon PoS chain ID 137."))}
    let native=rpc_string_params(&client,&endpoint,"eth_getBalance",json!([address,"latest"])).await?;
    let native_balance=format_units(&parse_big_quantity(&native)?,18);
    let mut tokens=Vec::with_capacity(token_addresses.len());
    for token in token_addresses {
        let code=rpc_string_params(&client,&endpoint,"eth_getCode",json!([token,"latest"])).await?;
        if code=="0x" || code=="0x0" {return Err(format!("No contract code was found at token address {token} on Polygon."))}
        let symbol_data=eth_call(&client,&endpoint,&token,"0x95d89b41".to_string()).await?;
        let decimals_data=eth_call(&client,&endpoint,&token,"0x313ce567".to_string()).await?;
        let decimals_raw=parse_quantity(&decimals_data)?;
        if decimals_raw>36{return Err(format!("Token {token} reports unsupported decimals."))}
        let decimals=decimals_raw as u8;
        let balance_data=eth_call(&client,&endpoint,&token,format!("0x70a08231{:0>64}",&address[2..].to_lowercase())).await?;
        let symbol=decode_abi_string(&symbol_data)?;
        let balance=format_units(&parse_big_quantity(&balance_data)?,decimals);
        tokens.push(TokenBalance{address:token,symbol,decimals,balance});
    }
    Ok(WalletBalances{address,native_balance,tokens})
}
#[tauri::command]
pub async fn read_wallet_balances(token_addresses:Vec<String>,state:State<'_,crate::AppState>)->Result<WalletBalances,String>{
    crate::require_authenticated(&state)?;
    let address=crate::wallet::stored_address()?.ok_or_else(||"Import a wallet before reading balances.".to_string())?;
    let result=read_saved_wallet_balances(address,token_addresses).await?;
    crate::require_authenticated(&state)?;
    Ok(result)
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
    if digits.is_empty() || !digits.bytes().all(|b|b.is_ascii_hexdigit()){return Err("The Polygon RPC provider returned an invalid hexadecimal quantity.".into())}
    let significant=digits.trim_start_matches('0');
    if significant.len()>16{return Err("The Polygon RPC provider returned an invalid hexadecimal quantity.".into())}
    if significant.is_empty(){return Ok(0)}
    u64::from_str_radix(significant,16).map_err(|_|"The Polygon RPC provider returned an invalid hexadecimal quantity.".into())
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
    #[test]fn validates_json_rpc_response_envelope(){
        assert_eq!(parse_rpc_result(br#"{"jsonrpc":"2.0","id":1,"result":"0x89"}"#).unwrap(),"0x89");
        assert!(parse_rpc_result(br#"{"jsonrpc":"2.0","id":2,"result":"0x89"}"#).is_err());
        assert!(parse_rpc_result(br#"{"jsonrpc":"1.0","id":1,"result":"0x89"}"#).is_err());
        assert!(parse_rpc_result(br#"{"jsonrpc":"2.0","id":1,"error":{"code":-1,"message":"failed"}}"#).is_err());
        assert!(parse_rpc_result(br#"{"jsonrpc":"2.0","id":1}"#).is_err());
        assert!(parse_rpc_result(b"not json").is_err());
    }

    #[test]fn big_integer_and_unit_formatting_are_exact(){
        assert_eq!(parse_big_quantity("0x0").unwrap(),"0");
        assert_eq!(parse_big_quantity("0xff").unwrap(),"255");
        assert_eq!(parse_big_quantity("0xde0b6b3a7640000").unwrap(),"1000000000000000000");
        assert_eq!(format_units("1000000000000000000",18),"1");
        assert_eq!(format_units("1234500",6),"1.2345");
        assert_eq!(format_units("12",4),"0.0012");
    }
    #[test]fn token_address_validation_rejects_malformed_values(){
        assert!(valid_evm_address(&format!("0x{}","a".repeat(40))));
        assert!(!valid_evm_address("0x1234"));
        assert!(!valid_evm_address(&format!("0x{}","g".repeat(40))));
    }
    #[test]fn decodes_erc20_symbol_abi_values(){
        assert_eq!(decode_abi_string("0x00000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000004555344430000000000000000000000000000000000000000000000000000000000").unwrap(),"USDC");
        assert_eq!(decode_abi_string("0x5553444300000000000000000000000000000000000000000000000000000000").unwrap(),"USDC");
        assert!(decode_abi_string("0x").is_err());
    }
    #[test]fn quantity_parser_checks_hex_and_bounds(){
        assert_eq!(parse_quantity("0x89").unwrap(),137);assert_eq!(parse_quantity("0x0").unwrap(),0);assert_eq!(parse_quantity(&format!("0x{}12","0".repeat(62))).unwrap(),18);
        assert!(parse_quantity("137").is_err());assert!(parse_quantity("0x").is_err());assert!(parse_quantity("0x10000000000000000").is_err());
    }
}