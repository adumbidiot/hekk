use std::ffi::OsString;
use winreg::{
    enums::{
        HKEY_LOCAL_MACHINE,
        KEY_ALL_ACCESS,
        KEY_ENUMERATE_SUB_KEYS,
    },
    RegKey,
};

/// A Registry Adapter
#[derive(Debug)]
pub struct RegistryAdapter {
    key: RegKey,
}

impl RegistryAdapter {
    pub const REGISTRY_ADAPTER_KEY_STR: &'static str =
        "SYSTEM\\CurrentControlSet\\Control\\Class\\{4D36E972-E325-11CE-BFC1-08002bE10318}";
    pub const HW_ADDRESS_KEY: &'static str = "NetworkAddress";

    /// Make a registry adapter from a key.
    pub fn from_key(key: RegKey) -> Self {
        RegistryAdapter { key }
    }

    /// Get the human readable name of this adapter
    pub fn get_description(&self) -> std::io::Result<String> {
        self.key.get_value("DriverDesc")
    }

    /// Get the name of the adapter. This is a guid.
    pub fn get_name(&self) -> std::io::Result<String> {
        self.key.get_value("NetCfgInstanceId")
    }

    /// Returns `None` if the hardware address does not exist.
    pub fn get_hardware_address(&self) -> std::io::Result<Option<OsString>> {
        match self.key.get_value(Self::HW_ADDRESS_KEY) {
            Ok(addr) => Ok(Some(addr)),
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Set the hardware address.
    ///
    /// Pass `None` to delete the registry key and reset the hardware address to its default.
    /// Hardware addresses can be XX-XX-XX-XX-XX-XX or xx-xx-xx-xx-xx-xx where X is a capital alphanumeric
    /// between A and F and x is a lowercase alphanumeric between a and f.
    /// This can also accept addresses in the form xxxxxxxxxxxx or XXXXXXXXXXXX.
    pub fn set_hardware_address(&self, hardware_address: Option<&str>) -> std::io::Result<()> {
        match hardware_address {
            Some(hardware_address) => self.key.set_value(Self::HW_ADDRESS_KEY, &hardware_address),
            None => {
                match self.key.delete_value(Self::HW_ADDRESS_KEY) {
                    Ok(()) => Ok(()),
                    Err(e) => {
                        // It is already unset
                        if e.kind() == std::io::ErrorKind::NotFound {
                            Ok(())
                        } else {
                            Err(e)
                        }
                    }
                }
            }
        }
    }

    /// Get the list of registry adapters in this system.
    ///
    /// This will try to get a read-only view of the adpater list, but writable adpaters.
    /// You need admin access for this to work properly.
    pub fn get_all() -> std::io::Result<Vec<std::io::Result<RegistryAdapter>>> {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let main_key =
            hklm.open_subkey_with_flags(Self::REGISTRY_ADAPTER_KEY_STR, KEY_ENUMERATE_SUB_KEYS)?;
        let iter = main_key.enum_keys();
        let keys = iter
            .filter_map(|key| match key {
                Ok(key) => {
                    // According to windows docs, adapter's registry key names are always 4 bytes in the form "xxxx".
                    if key.len() != 4 {
                        None
                    } else {
                        Some(Ok(key))
                    }
                }
                Err(e) => Some(Err(e)),
            })
            .map(|key_str| {
                main_key
                    .open_subkey_with_flags(&key_str?, KEY_ALL_ACCESS)
                    .map(RegistryAdapter::from_key)
            })
            .collect();

        Ok(keys)
    }
}
