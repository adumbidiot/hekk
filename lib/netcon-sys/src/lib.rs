#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

//  Good Resource: https://doxygen.reactos.org/de/d25/otherguids_8c.html

use winapi::{
    shared::{
        guiddef::{
            CLSID,
            GUID,
        },
        minwindef::DWORD,
        ntdef::{
            LPCWSTR,
            LPWSTR,
            ULONG,
        },
        winerror::HRESULT,
    },
    um::unknwnbase::{
        IUnknown,
        IUnknownVtbl,
    },
    DEFINE_GUID,
    ENUM,
    RIDL,
    STRUCT,
};

ENUM! {
    enum NETCONMGR_ENUM_FLAGS {
        NCME_DEFAULT = 0,
        NCME_HIDDEN = 1,
    }
}

ENUM! {
    enum NETCON_CHARACTERISTIC_FLAGS {
        NCCF_NONE = 0,
        NCCF_ALL_USERS = 0x1,
        NCCF_ALLOW_DUPLICATION = 0x2,
        NCCF_ALLOW_REMOVAL = 0x4,
        NCCF_ALLOW_RENAME = 0x8,
        NCCF_SHOW_ICON = 0x10,
        NCCF_INCOMING_ONLY = 0x20,
        NCCF_OUTGOING_ONLY = 0x40,
        NCCF_BRANDED = 0x80,
        NCCF_SHARED = 0x100,
        NCCF_BRIDGED = 0x200,
        NCCF_FIREWALLED = 0x400,
        NCCF_DEFAULT = 0x800,
        NCCF_HOMENET_CAPABLE = 0x1000,
        NCCF_SHARED_PRIVATE = 0x2000,
        NCCF_QUARANTINED = 0x4000,
        NCCF_RESERVED = 0x8000,
        NCCF_BLUETOOTH_MASK = 0xf0000,
        NCCF_LAN_MASK = 0xf00000,
    }
}

ENUM! {
    enum NETCON_STATUS {
        NCS_DISCONNECTED = 0,
        NCS_CONNECTING = 1,
        NCS_CONNECTED = 2,
        NCS_DISCONNECTING = 3,
        NCS_HARDWARE_NOT_PRESENT = 4,
        NCS_HARDWARE_DISABLED = 5,
        NCS_HARDWARE_MALFUNCTION = 6,
        NCS_MEDIA_DISCONNECTED = 7,
        NCS_AUTHENTICATING = 8,
        NCS_AUTHENTICATION_SUCCEEDED = 9,
        NCS_AUTHENTICATION_FAILED = 10,
        NCS_INVALID_ADDRESS = 11,
        NCS_CREDENTIALS_REQUIRED = 12,
    }
}

ENUM! {
    enum NETCON_TYPE {
        NCT_DIRECT_CONNECT = 0,
        NCT_INBOUND = 1,
        NCT_INTERNET = 2,
        NCT_LAN = 3,
        NCT_PHONE = 4,
        NCT_TUNNEL = 5,
        NCT_BRIDGE = 6,
    }
}

ENUM! {
    enum NETCON_MEDIATYPE {
        NCM_NONE = 0,
        NCM_DIRECT = 1,
        NCM_ISDN = 2,
        NCM_LAN = 3,
        NCM_PHONE = 4,
        NCM_TUNNEL = 5,
        NCM_PPPOE = 6,
        NCM_BRIDGE = 7,
        NCM_SHAREDACCESSHOST_LAN = 8,
        NCM_SHAREDACCESSHOST_RAS = 9,
  }
}

STRUCT! {
    struct NETCON_PROPERTIES {
        guidId: GUID,
        pszwName: LPWSTR,
        pszwDeviceName: LPWSTR,
        Status: NETCON_STATUS,
        MediaType: NETCON_MEDIATYPE,
        dwCharacter: DWORD,
        clsidThisObject: CLSID,
        clsidUiObject: CLSID,
    }
}

RIDL! {
    #[uuid(0xC08956A2, 0x1CD3, 0x11D1, 0xB1, 0xC5, 0x00, 0x80, 0x5F, 0xC1, 0x27, 0x0E)]
    interface INetConnectionManager(INetConnectionManagerVtbl): IUnknown(IUnknownVtbl) {
        fn EnumConnections(
            flags: NETCONMGR_ENUM_FLAGS,
            ptr: *mut *mut IEnumNetConnection,
        ) -> HRESULT,
    }
}

DEFINE_GUID! {
    CLSID_ConnectionManager,
    0xBA126AD1, 0x2166, 0x11D1, 0xB1, 0xD0, 0x0, 0x80, 0x5F, 0xC1, 0x27, 0x0E
}

RIDL! {
    #[uuid(0xC08956A0, 0x1CD3, 0x11D1, 0xB1, 0xC5, 0x00, 0x80, 0x5F, 0xC1, 0x27, 0x0E)]
    interface IEnumNetConnection(IEnumNetConnectionVtbl): IUnknown(IUnknownVtbl) {
        fn Next(
            celt: ULONG ,
            ptr: *mut *mut INetConnection,
            pceltFetched: *mut ULONG,
        ) -> HRESULT,
        fn Skip(
            celt: ULONG,
        ) -> HRESULT,
        fn Reset() -> HRESULT,
        fn Clone(
            ppenum: *mut *mut IEnumNetConnection,
        ) -> HRESULT,
    }
}

RIDL! {
    #[uuid(0xC08956A1, 0x1CD3, 0x11D1, 0xB1, 0xC5, 0x00, 0x80, 0x5F, 0xC1, 0x27, 0x0E)]
    interface INetConnection(INetConnectionVtbl): IUnknown(IUnknownVtbl) {
        fn Connect() -> HRESULT,
        fn Disconnect() -> HRESULT,
        fn Delete() -> HRESULT,
        fn Duplicate(
            pszwDuplicateName: LPCWSTR,
            ppCon: *mut *mut INetConnection,
        ) -> HRESULT,
        fn GetProperties(
            ppProps: *mut *mut NETCON_PROPERTIES,
        ) -> HRESULT,
        fn GetUiObjectClassId(
            pclsid: *mut CLSID,
        ) -> HRESULT,
        fn Rename(
            pszwNewName: LPCWSTR,
        ) -> HRESULT,
    }
}
