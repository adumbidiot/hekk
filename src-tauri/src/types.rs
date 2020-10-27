#[derive(Debug, serde::Serialize)]
pub struct AdapterInfo {
    pub name: String,
    pub description: String,

    #[serde(rename = "comboIndex")]
    pub combo_index: u32,

    pub address: Vec<u8>,

    #[serde(rename = "ipAddresses")]
    pub ip_addresses: Vec<IpAddress>,

    pub gateways: Vec<IpAddress>,
}

impl<'a> From<&'a iphlpapi::IpAdapterInfo> for AdapterInfo {
    fn from(adapter: &'a iphlpapi::IpAdapterInfo) -> Self {
        AdapterInfo {
            name: adapter.get_name().to_string_lossy().into_owned(),
            description: adapter.get_description().to_string_lossy().into_owned(),
            combo_index: adapter.get_combo_index(),
            address: adapter.get_address().to_vec(),
            ip_addresses: adapter
                .get_ip_address_list()
                .iter()
                .map(IpAddress::from)
                .collect(),
            gateways: adapter
                .get_ip_address_list()
                .iter()
                .map(IpAddress::from)
                .collect(),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct IpAddress {
    pub address: String,
    pub mask: String,
}

impl<'a> From<&'a iphlpapi::IpAddrString> for IpAddress {
    fn from(addr: &'a iphlpapi::IpAddrString) -> Self {
        Self {
            address: addr.get_address().to_string_lossy().into_owned(),
            mask: addr.get_mask().to_string_lossy().into_owned(),
        }
    }
}
