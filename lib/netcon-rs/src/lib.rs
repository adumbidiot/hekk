use bitflags::bitflags;
use netcon_sys::{
    CLSID_ConnectionManager,
    IEnumNetConnection,
    INetConnection,
    INetConnectionManager,
    NCCF_ALLOW_DUPLICATION,
    NCCF_ALLOW_REMOVAL,
    NCCF_ALLOW_RENAME,
    NCCF_ALL_USERS,
    NCCF_BLUETOOTH_MASK,
    NCCF_BRANDED,
    NCCF_BRIDGED,
    NCCF_DEFAULT,
    NCCF_FIREWALLED,
    NCCF_HOMENET_CAPABLE,
    NCCF_INCOMING_ONLY,
    NCCF_LAN_MASK,
    NCCF_NONE,
    NCCF_OUTGOING_ONLY,
    NCCF_QUARANTINED,
    NCCF_RESERVED,
    NCCF_SHARED,
    NCCF_SHARED_PRIVATE,
    NCCF_SHOW_ICON,
    NCME_DEFAULT,
    NCM_BRIDGE,
    NCM_DIRECT,
    NCM_ISDN,
    NCM_LAN,
    NCM_NONE,
    NCM_PHONE,
    NCM_PPPOE,
    NCM_SHAREDACCESSHOST_LAN,
    NCM_SHAREDACCESSHOST_RAS,
    NCM_TUNNEL,
    NCS_AUTHENTICATING,
    NCS_AUTHENTICATION_FAILED,
    NCS_AUTHENTICATION_SUCCEEDED,
    NCS_CONNECTED,
    NCS_CONNECTING,
    NCS_CREDENTIALS_REQUIRED,
    NCS_DISCONNECTED,
    NCS_DISCONNECTING,
    NCS_HARDWARE_DISABLED,
    NCS_HARDWARE_MALFUNCTION,
    NCS_HARDWARE_NOT_PRESENT,
    NCS_INVALID_ADDRESS,
    NCS_MEDIA_DISCONNECTED,
    NETCON_PROPERTIES,
};
use skylight::HResult;
use std::{
    convert::{
        TryFrom,
        TryInto,
    },
    ffi::OsString,
    fmt::Debug,
    os::windows::ffi::OsStringExt,
    ptr::NonNull,
};
use winapi::{
    shared::{
        guiddef::{
            CLSID,
            GUID,
        },
        winerror::{
            FAILED,
            S_FALSE,
            S_OK,
        },
        wtypesbase::{
            CLSCTX_LOCAL_SERVER,
            CLSCTX_NO_CODE_DOWNLOAD,
        },
    },
    um::{
        combaseapi::CoTaskMemFree,
        winbase::lstrlenW,
    },
};

/// A Manager for network connections
#[repr(transparent)]
pub struct NetConnectionManager(NonNull<INetConnectionManager>);

impl NetConnectionManager {
    /// Make a new [`NetConnectionManager`].
    pub fn new() -> Result<Self, HResult> {
        let ptr: *mut INetConnectionManager = unsafe {
            skylight::create_instance(
                &CLSID_ConnectionManager,
                CLSCTX_LOCAL_SERVER | CLSCTX_NO_CODE_DOWNLOAD,
            )?
        };

        Ok(NetConnectionManager(
            NonNull::new(ptr).expect("ptr was null"),
        ))
    }

    /// Iterate over [`NetConnection`]s.
    pub fn iter(&self) -> Result<EnumNetConnection, HResult> {
        let mut ptr = std::ptr::null_mut();
        let code = unsafe { self.0.as_ref().EnumConnections(NCME_DEFAULT, &mut ptr) };
        if FAILED(code) {
            return Err(HResult::from(code));
        }
        let ptr = NonNull::new(ptr).expect("ptr is null");
        Ok(EnumNetConnection(ptr))
    }
}

impl Drop for NetConnectionManager {
    fn drop(&mut self) {
        unsafe {
            self.0.as_ref().Release();
        }
    }
}

/// An enumerator over [`NetConnection`]s.
#[repr(transparent)]
pub struct EnumNetConnection(NonNull<IEnumNetConnection>);

impl EnumNetConnection {
    // TODO: Allow the user to generically specify how many items they want instead of only retrieving one at a time.
    /// Get the next connection.
    pub fn next_connection(&self) -> Result<Option<NetConnection>, HResult> {
        let mut ptr = std::ptr::null_mut();
        let mut num_recieved = 0;
        let ret = unsafe { self.0.as_ref().Next(1, &mut ptr, &mut num_recieved) };

        // Enumerators return S_FALSE to signal that there are no items left.
        match (ret, num_recieved) {
            (S_OK, 0) => Ok(None), // no items returned, yet successful. Assume no more items.
            (S_OK, _) => Ok(Some(NetConnection(NonNull::new(ptr).expect("ptr is null")))),
            (S_FALSE, _) => Ok(None), // the enumerator has signaled that it is done. num_recieved should be 0.
            _ => Err(HResult::from(ret)), // An error occured.
        }
    }

    /// Skip connections
    pub fn skip_connection(&self, n: u32) -> Result<(), HResult> {
        let code = unsafe { self.0.as_ref().Skip(n) };
        if FAILED(code) {
            return Err(HResult::from(code));
        }
        Ok(())
    }

    /// Reset this object
    pub fn reset(&self) -> Result<(), HResult> {
        let code = unsafe { self.0.as_ref().Reset() };
        if FAILED(code) {
            return Err(HResult::from(code));
        }
        Ok(())
    }
}

impl Iterator for EnumNetConnection {
    type Item = Result<NetConnection, HResult>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_connection().transpose()
    }
}

impl Drop for EnumNetConnection {
    fn drop(&mut self) {
        unsafe {
            self.0.as_ref().Release();
        }
    }
}

/// A network connection
#[repr(transparent)]
pub struct NetConnection(NonNull<INetConnection>);

impl NetConnection {
    /// Get the properties for this connection.
    pub fn get_properties(&self) -> Result<NetConProperties, HResult> {
        let mut ptr = std::ptr::null_mut();
        let code = unsafe { self.0.as_ref().GetProperties(&mut ptr) };

        if FAILED(code) {
            return Err(HResult::from(code));
        }

        Ok(NetConProperties(NonNull::new(ptr).expect("ptr is null")))
    }

    /// Connect a connection.
    pub fn connect(&self) -> Result<(), HResult> {
        let code = unsafe { self.0.as_ref().Connect() };
        if FAILED(code) {
            return Err(HResult::from(code));
        }
        Ok(())
    }

    /// Disconnect a connection.
    pub fn disconnect(&self) -> Result<(), HResult> {
        let code = unsafe { self.0.as_ref().Disconnect() };
        if FAILED(code) {
            return Err(HResult::from(code));
        }
        Ok(())
    }
}

impl std::fmt::Debug for NetConnection {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("NetConnection")
            .field("properties", &self.get_properties())
            .finish()
    }
}

impl Drop for NetConnection {
    fn drop(&mut self) {
        unsafe {
            self.0.as_ref().Release();
        }
    }
}

/// Network Connection Properties
#[repr(transparent)]
pub struct NetConProperties(NonNull<NETCON_PROPERTIES>);

impl NetConProperties {
    /// Get the GUID.
    pub fn guid(&self) -> &GUID {
        unsafe { &self.0.as_ref().guidId }
    }

    /// Get the raw name.
    ///
    /// # Panics
    /// Panics if the length is greater that a `usize`.
    pub fn raw_name(&self) -> &[u16] {
        unsafe {
            let ptr = self.0.as_ref().pszwName;
            let len = lstrlenW(ptr).try_into().expect("len cannot fit in a usize");
            std::slice::from_raw_parts(ptr, len)
        }
    }

    /// Get the name
    pub fn name(&self) -> OsString {
        OsString::from_wide(self.raw_name())
    }

    /// Get the raw device name.
    ///
    /// # Panics
    /// Panics if the length is greater that a `usize`.
    pub fn raw_device_name(&self) -> &[u16] {
        unsafe {
            let ptr = self.0.as_ref().pszwDeviceName;
            let len = lstrlenW(ptr).try_into().expect("len cannot fit in a usize");
            std::slice::from_raw_parts(ptr, len)
        }
    }

    /// Get the device name
    pub fn device_name(&self) -> OsString {
        OsString::from_wide(self.raw_device_name())
    }

    /// Get the raw connection status
    pub fn raw_status(&self) -> u32 {
        unsafe { self.0.as_ref().Status }
    }

    /// Get the connection status
    pub fn status(&self) -> Result<NetConStatus, u32> {
        NetConStatus::try_from(self.raw_status())
    }

    /// Get the raw media type for this connection
    pub fn raw_media_type(&self) -> u32 {
        unsafe { self.0.as_ref().MediaType }
    }

    /// Get the media_type of this connection
    pub fn media_type(&self) -> Result<MediaType, u32> {
        MediaType::try_from(self.raw_media_type())
    }

    /// Get the raw connection characteristics
    pub fn raw_characteristics(&self) -> u32 {
        unsafe { self.0.as_ref().dwCharacter }
    }

    /// Get the connection characteristics
    pub fn characteristics(&self) -> Option<NetConCharacteristicFlags> {
        NetConCharacteristicFlags::from_bits(self.raw_characteristics())
    }

    /// Get the clsid for this object
    pub fn clsid_this_object(&self) -> &CLSID {
        unsafe { &self.0.as_ref().clsidThisObject }
    }

    /// Get the clsid for the ui object
    pub fn clsid_ui_object(&self) -> &CLSID {
        unsafe { &self.0.as_ref().clsidUiObject }
    }
}

impl std::fmt::Debug for NetConProperties {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("NetConProperties")
            // .field("guid", &self.guid())
            .field("name", &self.name())
            .field("device_name", &self.device_name())
            .field("status", &self.status())
            .field("media_type", &self.media_type())
            .field("characteristics", &self.characteristics())
            // .field("clsid_this_object", &self.clsid_this_object())
            // .field("clsid_ui_object", &self.clsid_ui_object())
            .finish()
    }
}

impl Drop for NetConProperties {
    fn drop(&mut self) {
        unsafe {
            CoTaskMemFree(self.0.as_ref().pszwName.cast());
            CoTaskMemFree(self.0.as_ref().pszwDeviceName.cast());
            CoTaskMemFree(self.0.as_ptr().cast());
        }
    }
}

/// The connection status
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum NetConStatus {
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
    HardwareNotPresent,
    HardwareDisabled,
    HardwareMalfunction,
    MediaDisconnected,
    Authenticating,
    AuthenticationSucceeded,
    AuthenticationFailed,
    InvalidAddress,
    CredentialsRequired,
}

impl TryFrom<u32> for NetConStatus {
    type Error = u32;
    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            NCS_DISCONNECTED => Ok(NetConStatus::Disconnected),
            NCS_CONNECTING => Ok(NetConStatus::Connecting),
            NCS_CONNECTED => Ok(NetConStatus::Connected),
            NCS_DISCONNECTING => Ok(NetConStatus::Disconnecting),
            NCS_HARDWARE_NOT_PRESENT => Ok(NetConStatus::HardwareNotPresent),
            NCS_HARDWARE_DISABLED => Ok(NetConStatus::HardwareDisabled),
            NCS_HARDWARE_MALFUNCTION => Ok(NetConStatus::HardwareMalfunction),
            NCS_MEDIA_DISCONNECTED => Ok(NetConStatus::MediaDisconnected),
            NCS_AUTHENTICATING => Ok(NetConStatus::Authenticating),
            NCS_AUTHENTICATION_SUCCEEDED => Ok(NetConStatus::AuthenticationSucceeded),
            NCS_AUTHENTICATION_FAILED => Ok(NetConStatus::AuthenticationFailed),
            NCS_INVALID_ADDRESS => Ok(NetConStatus::InvalidAddress),
            NCS_CREDENTIALS_REQUIRED => Ok(NetConStatus::CredentialsRequired),
            _ => Err(v),
        }
    }
}

/// The Connection Type
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum MediaType {
    None,
    Direct,
    Isdn,
    Lan,
    Phone,
    Tunnel,
    Pppoe,
    Bridge,
    SharedAccessHostLan,
    SharedAccessHostRas,
}

impl TryFrom<u32> for MediaType {
    type Error = u32;
    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            NCM_NONE => Ok(MediaType::None),
            NCM_DIRECT => Ok(MediaType::Direct),
            NCM_ISDN => Ok(MediaType::Isdn),
            NCM_LAN => Ok(MediaType::Lan),
            NCM_PHONE => Ok(MediaType::Phone),
            NCM_TUNNEL => Ok(MediaType::Tunnel),
            NCM_PPPOE => Ok(MediaType::Pppoe),
            NCM_BRIDGE => Ok(MediaType::Bridge),
            NCM_SHAREDACCESSHOST_LAN => Ok(MediaType::SharedAccessHostLan),
            NCM_SHAREDACCESSHOST_RAS => Ok(MediaType::SharedAccessHostRas),
            _ => Err(v),
        }
    }
}

bitflags! {
    pub struct NetConCharacteristicFlags: u32 {
        const NONE = NCCF_NONE;
        const ALL_USERS = NCCF_ALL_USERS;
        const ALLOW_DUPLICATION = NCCF_ALLOW_DUPLICATION;
        const ALLOW_REMOVAL = NCCF_ALLOW_REMOVAL;
        const ALLOW_RENAME = NCCF_ALLOW_RENAME;
        const SHOW_ICON = NCCF_SHOW_ICON;
        const INCOMING_ONLY = NCCF_INCOMING_ONLY;
        const OUTGOING_ONLY = NCCF_OUTGOING_ONLY;
        const BRANDED = NCCF_BRANDED;
        const SHARED = NCCF_SHARED;
        const BRIDGED = NCCF_BRIDGED;
        const FIREWALLED = NCCF_FIREWALLED;
        const DEFAULT = NCCF_DEFAULT;
        const HOMENET_CAPABLE = NCCF_HOMENET_CAPABLE;
        const SHARED_PRIVATE = NCCF_SHARED_PRIVATE;
        const QUARANTINED = NCCF_QUARANTINED;
        const RESERVED = NCCF_RESERVED;
        const BLUETOOTH_MASK = NCCF_BLUETOOTH_MASK;
        const LAN_MASK = NCCF_LAN_MASK;
    }
}
