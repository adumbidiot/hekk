use clap::{
    App,
    ArgMatches,
    SubCommand,
};
use iphlpapi::IpAdapterInfo;

pub fn cli() -> App<'static, 'static> {
    SubCommand::with_name("adapter-info").about("Gets adapter info")
}

pub fn exec(_matches: &ArgMatches) {
    let adapters = match iphlpapi::get_adapters_info() {
        Ok(v) => v,
        Err(e) => {
            println!("Failed to get adapters, got error: {:#?}", e);
            return;
        }
    };

    for adapter in adapters.iter() {
        println!("{}", IpAdapterInfoDisplayRef(&adapter));
        println!("-----");
        println!();
    }
}

pub struct IpAdapterInfoDisplayRef<'a>(&'a IpAdapterInfo);

impl<'a> std::fmt::Display for IpAdapterInfoDisplayRef<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Adapter Name: {}", self.0.get_name().to_string_lossy())?;

        writeln!(
            f,
            "Description: {}",
            self.0.get_description().to_string_lossy()
        )?;

        writeln!(f, "Combo Index: {}", self.0.get_combo_index())?;

        write!(f, "Hardware Address: ")?;
        let addresses = self.0.get_address();
        for (i, b) in addresses.iter().enumerate() {
            if i == addresses.len() - 1 {
                write!(f, "{:02X}", b)?;
            } else {
                write!(f, "{:02X}-", b)?;
            }
        }
        writeln!(f)?;

        writeln!(f, "IP Address List: ")?;
        for ip in self.0.get_ip_address_list().iter() {
            writeln!(f, "  Ip: {}", ip.get_address().to_string_lossy())?;
            writeln!(f, "  Mask: {}", ip.get_mask().to_string_lossy())?;
            writeln!(f)?;
        }

        writeln!(f, "Gateway List: ")?;
        for ip in self.0.get_gateway_list().iter() {
            writeln!(f, "  Ip: {}", ip.get_address().to_string_lossy())?;
            writeln!(f, "  Mask: {}", ip.get_mask().to_string_lossy())?;
            writeln!(f,)?;
        }

        Ok(())
    }
}
