use k256::{ecdsa::SigningKey, elliptic_curve::sec1::ToEncodedPoint};
use sha3::{Digest, Keccak256};
use zeroize::Zeroizing;

const KEYRING_SERVICE: &str = "com.arbx.desktop";
const KEYRING_USER: &str = "polygon-mainnet-wallet";

fn decode_private_key(input: &str) -> Result<Zeroizing<Vec<u8>>, String> {
    let value = input.trim();
    let value = value.strip_prefix("0x").or_else(|| value.strip_prefix("0X")).unwrap_or(value);
    if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("The private key must contain exactly 64 hexadecimal characters.".into());
    }
    let mut bytes = Zeroizing::new(vec![0u8; 32]);
    hex::decode_to_slice(value, &mut bytes[..])
        .map_err(|_| "The private key is not valid hexadecimal.".to_string())?;
    SigningKey::from_slice(&bytes)
        .map_err(|_| "The private key is not a valid secp256k1 scalar.".to_string())?;
    Ok(bytes)
}

fn derive_evm_address(private_key: &[u8]) -> Result<String, String> {
    let signing_key = SigningKey::from_slice(private_key)
        .map_err(|_| "The private key is not valid for an EVM wallet.".to_string())?;
    let public_key = signing_key.verifying_key().to_encoded_point(false);
    let digest = Keccak256::digest(&public_key.as_bytes()[1..]);
    Ok(format!("0x{}", hex::encode(&digest[12..])))
}

#[cfg(windows)]
fn prompt_for_private_key() -> Result<Zeroizing<String>, String> {
    use std::{ffi::c_void, mem::size_of, ptr::{null, null_mut}};
    use windows_sys::Win32::{
        Foundation::{GetLastError, ERROR_CANCELLED, ERROR_INSUFFICIENT_BUFFER},
        Security::Credentials::{CredUIPromptForWindowsCredentialsW, CredUnPackAuthenticationBufferW, CREDUI_INFOW, CREDUIWIN_GENERIC},
        System::Com::CoTaskMemFree,
    };
    fn wide(s: &str) -> Vec<u16> { s.encode_utf16().chain(std::iter::once(0)).collect() }
    struct AuthBuffer(*mut c_void, usize);
    impl Drop for AuthBuffer {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    std::ptr::write_bytes(self.0.cast::<u8>(), 0, self.1);
                    CoTaskMemFree(self.0);
                }
            }
        }
    }

    let caption = wide("Import wallet into ArbX");
    let message = wide("Enter your private key in the Password field. The username is ignored. The key is handled by the native application and is never sent to the web interface.");
    let ui = CREDUI_INFOW {
        cbSize: size_of::<CREDUI_INFOW>() as u32,
        hwndParent: null_mut(),
        pszMessageText: message.as_ptr(),
        pszCaptionText: caption.as_ptr(),
        hbmBanner: null_mut(),
    };
    let mut auth_package = 0u32;
    let mut auth_buffer: *mut c_void = null_mut();
    let mut auth_size = 0u32;
    let status = unsafe {
        CredUIPromptForWindowsCredentialsW(&ui, 0, &mut auth_package, null(), 0, &mut auth_buffer, &mut auth_size, null_mut(), CREDUIWIN_GENERIC)
    };
    if status == ERROR_CANCELLED { return Err("Wallet import was cancelled.".into()); }
    if status != 0 || auth_buffer.is_null() || auth_size == 0 {
        return Err("Windows could not open the secure wallet-entry prompt.".into());
    }
    let _guard = AuthBuffer(auth_buffer, auth_size as usize);
    let mut user_len = 0u32;
    let mut domain_len = 0u32;
    let mut pass_len = 0u32;
    let ok = unsafe {
        CredUnPackAuthenticationBufferW(0, auth_buffer, auth_size, null_mut(), &mut user_len, null_mut(), &mut domain_len, null_mut(), &mut pass_len)
    };
    if ok == 0 && unsafe { GetLastError() } != ERROR_INSUFFICIENT_BUFFER {
        return Err("Windows could not read the credential prompt result.".into());
    }
    if pass_len == 0 || pass_len > 2048 || user_len > 2048 || domain_len > 2048 {
        return Err("The secure prompt returned an invalid credential size.".into());
    }
    let mut username = Zeroizing::new(vec![0u16; user_len.max(1) as usize]);
    let mut domain = Zeroizing::new(vec![0u16; domain_len.max(1) as usize]);
    let mut password = Zeroizing::new(vec![0u16; pass_len as usize]);
    let ok = unsafe {
        CredUnPackAuthenticationBufferW(0, auth_buffer, auth_size, username.as_mut_ptr(), &mut user_len, domain.as_mut_ptr(), &mut domain_len, password.as_mut_ptr(), &mut pass_len)
    };
    if ok == 0 { return Err("Windows could not unpack the secure credential response.".into()); }
    let end = password.iter().position(|unit| *unit == 0).unwrap_or(password.len());
    let key = String::from_utf16(&password[..end])
        .map_err(|_| "The private key must be entered as hexadecimal text.".to_string())?;
    if key.is_empty() { return Err("No private key was entered.".into()); }
    Ok(Zeroizing::new(key))
}

#[cfg(not(windows))]
fn prompt_for_private_key() -> Result<Zeroizing<String>, String> {
    Err("Secure wallet import is currently supported only in the Windows desktop application.".into())
}

pub fn prompt_and_derive() -> Result<(String, Zeroizing<Vec<u8>>), String> {
    let entered = prompt_for_private_key()?;
    let key = decode_private_key(&entered)?;
    let address = derive_evm_address(&key)?;
    Ok((address, key))
}

#[cfg(windows)]
fn entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .map_err(|_| "Windows Credential Manager is unavailable.".to_string())
}

#[cfg(windows)]
pub fn replace_secret(secret: &[u8]) -> Result<Option<Zeroizing<Vec<u8>>>, String> {
    let credential = entry()?;
    let previous = match credential.get_secret() {
        Ok(value) => Some(Zeroizing::new(value)),
        Err(keyring::Error::NoEntry) => None,
        Err(_) => return Err("The existing wallet credential could not be read.".into()),
    };
    credential.set_secret(secret)
        .map_err(|_| "Windows Credential Manager could not store the wallet.".to_string())?;
    Ok(previous)
}

#[cfg(windows)]
pub fn restore_secret(previous: Option<&[u8]>) -> Result<(), String> {
    let credential = entry()?;
    match previous {
        Some(value) => credential.set_secret(value)
            .map_err(|_| "The previous wallet credential could not be restored.".to_string()),
        None => match credential.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(_) => Err("The newly stored wallet credential could not be removed.".into()),
        },
    }
}

#[cfg(windows)]
pub fn delete_secret() -> Result<Option<Zeroizing<Vec<u8>>>, String> {
    let credential = entry()?;
    let previous = match credential.get_secret() {
        Ok(value) => Some(Zeroizing::new(value)),
        Err(keyring::Error::NoEntry) => None,
        Err(_) => return Err("The wallet credential could not be read.".into()),
    };
    if previous.is_some() {
        credential.delete_credential()
            .map_err(|_| "Windows Credential Manager could not remove the wallet.".to_string())?;
    }
    Ok(previous)
}

#[cfg(windows)]
pub fn has_secret() -> Result<bool, String> {
    let credential = entry()?;
    match credential.get_secret() {
        Ok(value) => { let secret = Zeroizing::new(value); Ok(!secret.is_empty()) }
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(_) => Err("Windows Credential Manager could not verify the wallet credential.".into()),
    }
}

#[cfg(not(windows))]
pub fn replace_secret(_: &[u8]) -> Result<Option<Zeroizing<Vec<u8>>>, String> {
    Err("Windows Credential Manager is available only in the Windows desktop application.".into())
}
#[cfg(not(windows))]
pub fn restore_secret(_: Option<&[u8]>) -> Result<(), String> {
    Err("Windows Credential Manager is available only in the Windows desktop application.".into())
}
#[cfg(not(windows))]
pub fn delete_secret() -> Result<Option<Zeroizing<Vec<u8>>>, String> {
    Err("Windows Credential Manager is available only in the Windows desktop application.".into())
}
#[cfg(not(windows))]
pub fn has_secret() -> Result<bool, String> { Ok(false) }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_wrong_private_key_lengths() {
        assert!(decode_private_key("0x1234").is_err());
        assert!(decode_private_key(&"a".repeat(63)).is_err());
        assert!(decode_private_key(&"a".repeat(65)).is_err());
    }
    #[test]
    fn rejects_non_hexadecimal_private_keys() {
        assert!(decode_private_key(&"g".repeat(64)).is_err());
    }
    #[test]
    fn rejects_zero_scalar() {
        assert!(decode_private_key(&"0".repeat(64)).is_err());
    }
}
