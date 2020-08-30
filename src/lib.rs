use pnet::datalink::{self, MacAddr};
use std::fmt::{self, Display, Formatter};
use std::net::Ipv4Addr;

/// Represents a network interface and its associated addresses.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Interface {
    name: String,
    alias: Option<String>,
    hardware_addr: MacAddr,
    ip_addrs: Vec<Ipv4Addr>,
    mtu: usize,
    is_up: bool,
    is_loopback: bool,
}

impl Interface {
    /// Constructs a new empty `Interface`.
    pub fn new() -> Interface {
        Interface {
            name: String::new(),
            alias: None,
            hardware_addr: MacAddr::zero(),
            ip_addrs: vec![],
            mtu: 0,
            is_up: false,
            is_loopback: false,
        }
    }

    /// Returns the name of the interface.
    pub fn name(&self) -> &String {
        &self.name
    }

    /// Returns the alias of the interface.
    pub fn alias(&self) -> &Option<String> {
        &self.alias
    }

    /// Returns the hardware address of the interface.
    pub fn hardware_addr(&self) -> MacAddr {
        self.hardware_addr
    }

    /// Returns the first IPv4 address of the interface.
    pub fn ip_addr(&self) -> Option<Ipv4Addr> {
        if self.ip_addrs.len() > 0 {
            Some(self.ip_addrs[0])
        } else {
            None
        }
    }

    /// Returns the MTU of the interface.
    pub fn mtu(&self) -> usize {
        self.mtu
    }

    /// Returns if the interface is up.
    pub fn is_up(&self) -> bool {
        self.is_up
    }

    /// Returns if the interface is a loopback interface.
    pub fn is_loopback(&self) -> bool {
        self.is_loopback
    }
}

impl Display for Interface {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let name = match &self.alias {
            Some(alias) => format!("{} ({})", self.name, alias),
            None => self.name.clone(),
        };

        let ip_addrs = format!(
            "{}",
            self.ip_addrs
                .iter()
                .map(|ip_addr| { ip_addr.to_string() })
                .collect::<Vec<_>>()
                .join(", ")
        );

        let mut flags = String::new();
        if self.is_loopback {
            flags = String::from(" (Loopback)");
        }

        write!(
            f,
            "{} [{}]{}: {}",
            name, self.hardware_addr, flags, ip_addrs
        )
    }
}

/// Gets a list of available network interfaces for the current machine.
pub fn interfaces() -> Vec<Interface> {
    let inters = datalink::interfaces();

    let ifs = inters
        .iter()
        .map(|inter| {
            /* Cannot get flags using WinPcap in Windows
            if !inter.is_up() {
                return Err(());
            }
            */

            let mut i = Interface::new();
            i.name = inter.name.clone();
            i.hardware_addr = match inter.mac {
                Some(mac) => mac,
                None => return Err(()),
            };
            i.ip_addrs = inter
                .ips
                .iter()
                .map(|ip| match ip {
                    ipnetwork::IpNetwork::V4(ref ipv4) => Ok(ipv4.ip()),
                    _ => Err(()),
                })
                .filter_map(Result::ok)
                .collect();

            // Exclude interface without any IPv4 address
            if i.ip_addrs.len() <= 0 {
                return Err(());
            }

            i.is_up = inter.is_up();
            i.is_loopback = inter.is_loopback();

            Ok(i)
        })
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    let ifs = mark_interfaces(ifs);

    ifs
}

#[cfg(windows)]
fn mark_interfaces(mut ifs: Vec<Interface>) -> Vec<Interface> {
    if let Ok(sys_inters) = netifs::get_interfaces() {
        for inter in sys_inters {
            for i in &mut ifs {
                if i.name.ends_with(&inter.name) {
                    i.alias = Some(inter.display_name.clone());
                    i.mtu = inter.mtu;
                    i.is_up = inter.is_up;
                    i.is_loopback = inter.is_loopback;
                }
            }
        }
    }

    ifs
}

#[cfg(not(windows))]
fn mark_interfaces(mut ifs: Vec<Interface>) -> Vec<Interface> {
    if let Ok(sys_inters) = c_interfaces::Interface::get_all() {
        for inter in sys_inters {
            for i in &mut ifs {
                if i.name == inter.name {
                    if let Ok(mtu) = inter.get_mtu() {
                        i.mtu = mtu as usize;
                    }
                }
            }
        }
    }

    ifs
}
